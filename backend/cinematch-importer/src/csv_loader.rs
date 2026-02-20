use crate::parsing::deserializer::MovieDataBuilder;
use anyhow::Result;
use cinematch_db::conn::qdrant::models::MovieData;
use std::collections::VecDeque;
use std::io::BufReader;

/// Async streaming movie iterator with buffering for efficient memory usage.
///
/// Reads a CSV file lazily via `spawn_blocking`, yielding one `MovieData` at a
/// time with an internal buffer to amortise the blocking-call overhead.
pub struct AsyncMovieStream {
    /// CSV reader — moved into `spawn_blocking` on each refill, then moved back.
    reader: Option<csv::Reader<BufReader<std::fs::File>>>,
    buffer: VecDeque<MovieData>,
    buffer_size: usize,
    exhausted: bool,
}

impl AsyncMovieStream {
    /// Open the CSV file and pre-fill the buffer.
    pub async fn new(path: &str, buffer_size: usize) -> Result<Self> {
        let path = path.to_string();

        let reader = tokio::task::spawn_blocking({
            let path = path.clone();
            move || -> Result<csv::Reader<BufReader<std::fs::File>>> {
                let file = std::fs::File::open(&path)?;
                Ok(csv::Reader::from_reader(BufReader::new(file)))
            }
        })
        .await??;

        let mut stream = Self {
            reader: Some(reader),
            buffer: VecDeque::with_capacity(buffer_size),
            buffer_size,
            exhausted: false,
        };

        stream.fill_buffer().await?;
        Ok(stream)
    }

    /// Refill the internal buffer by reading up to `buffer_size` rows.
    async fn fill_buffer(&mut self) -> Result<()> {
        if self.exhausted {
            return Ok(());
        }

        if let Some(reader) = self.reader.take() {
            let buffer_size = self.buffer_size;

            let (reader, movies, exhausted) = tokio::task::spawn_blocking(move || {
                let mut movies = Vec::with_capacity(buffer_size);
                let mut exhausted = false;
                let mut reader = reader;

                let mut iter = reader.deserialize::<MovieDataBuilder>();
                for _ in 0..buffer_size {
                    match iter.next() {
                        Some(Ok(builder)) => movies.push(builder.build()),
                        Some(Err(e)) => {
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

            self.reader = Some(reader);
            self.exhausted = exhausted;
            self.buffer.extend(movies);
        }

        Ok(())
    }

    /// Yield the next movie, refilling the buffer when empty.
    pub async fn next_movie(&mut self) -> Result<Option<MovieData>> {
        if self.buffer.is_empty() && !self.exhausted {
            self.fill_buffer().await?;
        }
        Ok(self.buffer.pop_front())
    }
}

/// Open a streaming movie iterator with the default buffer size (64).
pub async fn load_and_preprocess_movies(path: &str) -> Result<AsyncMovieStream> {
    AsyncMovieStream::new(path, 64).await
}
