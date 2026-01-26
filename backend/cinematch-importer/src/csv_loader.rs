use crate::MovieData;
use crate::deserializer::MovieDataBuilder;
use anyhow::Result;
use indicatif::ProgressBar;
use std::collections::HashMap;
use std::io::BufReader;

/// Async streaming movie iterator with buffering for efficient memory usage
/// Single reader instance, no Arc/Mutex - accessed sequentially through spawn_blocking
/// Also supports random access by movie_id via lazy indexing
#[allow(dead_code)]
pub struct AsyncMovieStream {
    /// CSV reader - owned by this struct, moved into spawn_blocking when needed
    reader: Option<csv::Reader<BufReader<std::fs::File>>>,
    buffer: std::collections::VecDeque<MovieData>,
    buffer_size: usize,
    exhausted: bool,
    /// Index: movie_id -> ordinal position in stream (built lazily)
    id_index: HashMap<i64, usize>,
    /// Path to CSV file (needed for random access)
    file_path: String,
    /// Progress bar for tracking load progress
    progress: Option<ProgressBar>,
}

#[allow(dead_code)]
impl AsyncMovieStream {
    /// Create a new async streaming movie iterator with a buffer size
    /// Index is NOT built during initialization - call build_index() for that
    pub async fn new(path: &str, buffer_size: usize) -> Result<Self> {
        let path_str = path.to_string();

        // Open file and create reader in blocking task
        let reader = tokio::task::spawn_blocking({
            let path = path_str.clone();
            move || -> Result<csv::Reader<BufReader<std::fs::File>>> {
                let file = std::fs::File::open(&path)?;
                let buf_reader = BufReader::new(file);
                let reader = csv::Reader::from_reader(buf_reader);
                Ok(reader)
            }
        })
        .await??;

        let mut stream = AsyncMovieStream {
            reader: Some(reader),
            buffer: std::collections::VecDeque::with_capacity(buffer_size),
            buffer_size,
            exhausted: false,
            id_index: HashMap::new(),
            file_path: path_str,
            progress: None,
        };

        // Pre-fill the buffer
        stream.fill_buffer().await?;
        Ok(stream)
    }

    /// Build the index by reading each line and extracting just the movie_id
    /// This is called lazily when random access is needed
    /// Returns the number of movies indexed
    pub async fn build_index(&mut self) -> Result<usize> {
        let file_path = self.file_path.clone();
        let progress = self.progress.clone();

        let id_index = tokio::task::spawn_blocking(move || -> Result<HashMap<i64, usize>> {
            use std::fs::File;
            use std::io::{BufRead, BufReader};

            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);
            let mut id_index = HashMap::new();
            let mut line_num = 0;
            let mut skipped = 0;

            for line in reader.lines() {
                let line = line?;

                // Skip header line
                if line_num == 0 {
                    line_num += 1;
                    continue;
                }

                // Extract movie_id (first field before first comma)
                if let Some(id_str) = line.split(',').next() {
                    if let Ok(movie_id) = id_str.trim().parse::<i64>() {
                        id_index.insert(movie_id, line_num);
                        if let Some(ref pb) = progress {
                            pb.inc(1);
                        }
                    } else {
                        skipped += 1;
                    }
                } else {
                    skipped += 1;
                }

                line_num += 1;
            }

            if skipped > 0 {
                eprintln!(
                    "⚠️  Indexing complete: {} movies indexed, {} rows skipped",
                    id_index.len(),
                    skipped
                );
            }

            Ok(id_index)
        })
        .await??;

        let count = id_index.len();
        self.id_index = id_index;
        Ok(count)
    }

    /// Set a progress bar for tracking streaming progress
    /// Returns the progress bar that was previously set (if any)
    pub fn set_progress(&mut self, progress: Option<ProgressBar>) -> Option<ProgressBar> {
        std::mem::replace(&mut self.progress, progress)
    }

    /// Get the current progress bar (for external updates)
    pub fn progress(&self) -> Option<&ProgressBar> {
        self.progress.as_ref()
    }

    /// Get the number of total movies in the index (0 if not built yet)
    pub fn total_indexed(&self) -> usize {
        self.id_index.len()
    }

    /// Fill the buffer by moving reader into spawn_blocking, reading, then moving back
    async fn fill_buffer(&mut self) -> Result<()> {
        if self.exhausted {
            return Ok(());
        }

        let buffer_size = self.buffer_size;
        let progress = self.progress.clone();

        // Take reader from Option, read in blocking task, put back
        if let Some(reader) = self.reader.take() {
            let (reader, movies, exhausted) = tokio::task::spawn_blocking(move || {
                let mut movies = Vec::new();
                let mut exhausted = false;
                let mut reader = reader;

                let mut deserializer = reader.deserialize::<MovieDataBuilder>();
                for _ in 0..buffer_size {
                    match deserializer.next() {
                        Some(Ok(builder)) => {
                            let movie = builder.build();
                            if let Some(ref pb) = progress {
                                pb.inc(1);
                            }
                            movies.push(movie);
                        }
                        Some(Err(e)) => {
                            // Try to extract movie_id from the error context
                            eprintln!("⚠️  Failed to deserialize row: {}", e);
                        }
                        None => {
                            exhausted = true;
                            break;
                        }
                    }
                }

                (reader, movies, exhausted)
            })
            .await?;

            // Move reader back into self
            self.reader = Some(reader);
            self.exhausted = exhausted;

            for movie in movies {
                self.buffer.push_back(movie);
            }
        }

        Ok(())
    }

    /// Get the next movie from the stream (async method)
    pub async fn next_movie(&mut self) -> Result<Option<MovieData>> {
        // If buffer empty and not exhausted, refill it
        if self.buffer.is_empty() && !self.exhausted {
            self.fill_buffer().await?;
        }

        Ok(self.buffer.pop_front())
    }

    /// Get a specific movie by ID (random access)
    /// Uses the index to skip directly to the target line if available
    /// Falls back to O(n) scan if index not built
    pub async fn get(&self, movie_id: i64) -> Result<Option<MovieData>> {
        let file_path = self.file_path.clone();
        let target_line = self.id_index.get(&movie_id).copied();

        // Read specific movie in blocking task
        tokio::task::spawn_blocking(move || -> Result<Option<MovieData>> {
            use std::fs::File;
            use std::io::{BufRead, BufReader};

            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);

            if let Some(target_line_num) = target_line {
                // Use index to skip directly to the target line
                let mut lines = reader.lines();

                // Skip to target line (lines are 1-indexed from index)
                for _ in 0..target_line_num {
                    if lines.next().is_none() {
                        return Ok(None);
                    }
                }

                // Parse the target line
                if let Some(Ok(line)) = lines.next() {
                    let header_and_line = format!(
                        "{}\n{}",
                        // Read header from file
                        std::io::BufReader::new(File::open(&file_path)?)
                            .lines()
                            .next()
                            .and_then(|r| r.ok())
                            .unwrap_or_default(),
                        line
                    );

                    let mut reader = csv::Reader::from_reader(header_and_line.as_bytes());
                    if let Some(result) = reader.deserialize::<MovieDataBuilder>().next() {
                        match result {
                            Ok(builder) => {
                                if builder.movie_id == movie_id {
                                    return Ok(Some(builder.build()));
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️  Failed to deserialize movie {}: {}", movie_id, e);
                            }
                        }
                    }
                }
            } else {
                // Index not built, fall back to full scan
                let mut reader = csv::Reader::from_reader(reader);
                let mut deserializer = reader.deserialize::<MovieDataBuilder>();
                for result in &mut deserializer {
                    match result {
                        Ok(builder) => {
                            if builder.movie_id == movie_id {
                                return Ok(Some(builder.build()));
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️  Failed to deserialize movie {}: {}", movie_id, e);
                            return Ok(None);
                        }
                    }
                }
            }

            Ok(None)
        })
        .await?
    }

    /// Get multiple movies by their IDs (batch random access)
    /// Returns a map of movie_id -> MovieData
    pub async fn get_batch(&self, movie_ids: Vec<i64>) -> Result<HashMap<i64, MovieData>> {
        let file_path = self.file_path.clone();
        let id_set: std::collections::HashSet<_> = movie_ids.into_iter().collect();

        tokio::task::spawn_blocking(move || -> Result<HashMap<i64, MovieData>> {
            let file = std::fs::File::open(&file_path)?;
            let buf_reader = BufReader::new(file);
            let mut reader = csv::Reader::from_reader(buf_reader);

            let mut results = HashMap::new();
            let mut found_count = 0;
            let target_count = id_set.len();

            let mut deserializer = reader.deserialize::<MovieDataBuilder>();
            for result in &mut deserializer {
                match result {
                    Ok(builder) => {
                        if id_set.contains(&builder.movie_id) {
                            results.insert(builder.movie_id, builder.build());
                            found_count += 1;
                            if found_count == target_count {
                                break; // Found all requested movies
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to deserialize in batch lookup: {}", e);
                    }
                }
            }

            Ok(results)
        })
        .await?
    }
}

/// Load movies as an async streaming iterator with default buffer size of 5
pub async fn load_and_preprocess_movies(path: &str) -> Result<AsyncMovieStream> {
    AsyncMovieStream::new(path, 5).await
}

/// Load movies as an async streaming iterator with custom buffer size
#[allow(dead_code)]
pub async fn load_and_preprocess_movies_buffered(
    path: &str,
    buffer_size: usize,
) -> Result<AsyncMovieStream> {
    AsyncMovieStream::new(path, buffer_size).await
}
