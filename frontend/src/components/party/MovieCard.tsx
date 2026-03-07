import { useState, useRef, useCallback, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import { MovieResponse } from '@/model/movieResponse'
import { ThumbsDown, ThumbsUp, SkipForward, Star, Calendar, Clock } from 'lucide-react'
import Image from 'next/image'

const DISTANCE_THRESHOLD = 100
const VELOCITY_THRESHOLD = 0.5 // px/ms
const MIN_FLICK_DISTANCE = 30

interface TouchPoint {
  x: number
  y: number
  time: number
}

interface MovieCardProps {
  movie: MovieResponse
  onLike: () => void
  onDislike: () => void
  onSkip: () => void
  disabled?: boolean
}

export default function MovieCard({
  movie,
  onLike,
  onDislike,
  onSkip,
  disabled
}: Readonly<MovieCardProps>) {
  const [isImageLoading, setIsImageLoading] = useState(true)
  const [prevPosterUrl, setPrevPosterUrl] = useState(movie.poster_url)
  const [swipeState, setSwipeState] = useState<'idle' | 'dragging' | 'exiting'>('idle')

  const cardRef = useRef<HTMLDivElement>(null)
  const likeOverlayRef = useRef<HTMLDivElement>(null)
  const dislikeOverlayRef = useRef<HTMLDivElement>(null)
  const skipOverlayRef = useRef<HTMLDivElement>(null)

  const touchStartRef = useRef<{ x: number; y: number } | null>(null)
  const offsetRef = useRef({ x: 0, y: 0 })
  const touchHistoryRef = useRef<TouchPoint[]>([])
  const animFrameRef = useRef(0)
  const exitTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const swipeStateRef = useRef<'idle' | 'dragging' | 'exiting'>('idle')

  // Reset state when movie changes (adjusting state based on props — allowed during render)
  const [prevMovieId, setPrevMovieId] = useState(movie.movie_id)
  if (movie.movie_id !== prevMovieId) {
    setPrevMovieId(movie.movie_id)
    setSwipeState('idle')
  }

  // Reset DOM when movie changes
  useEffect(() => {
    if (cardRef.current) {
      cardRef.current.style.transition = ''
      cardRef.current.style.transform = ''
    }
    for (const ref of [likeOverlayRef, dislikeOverlayRef, skipOverlayRef]) {
      if (ref.current) {
        ref.current.style.transition = ''
        ref.current.style.opacity = '0'
      }
    }
    offsetRef.current = { x: 0, y: 0 }
    touchStartRef.current = null
    swipeStateRef.current = 'idle'
  }, [movie.movie_id])

  if (movie.poster_url !== prevPosterUrl) {
    setPrevPosterUrl(movie.poster_url)
    setIsImageLoading(true)
  }

  useEffect(() => {
    swipeStateRef.current = swipeState
  }, [swipeState])

  useEffect(() => {
    return () => {
      if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current)
      if (exitTimeoutRef.current) clearTimeout(exitTimeoutRef.current)
    }
  }, [])

  const updateCardDOM = useCallback((x: number, y: number) => {
    if (!cardRef.current) return
    cardRef.current.style.transform = `translate(${x}px, ${y}px) rotate(${x * 0.08}deg)`
  }, [])

  const updateOverlayDOM = useCallback((x: number, y: number) => {
    const absX = Math.abs(x)
    let likeOp = 0
    let dislikeOp = 0
    let skipOp = 0

    if (absX > 20 || y > 20) {
      if (absX > y) {
        const progress = Math.min(1, absX / DISTANCE_THRESHOLD)
        if (x > 0) likeOp = progress * 0.8
        else dislikeOp = progress * 0.8
      } else {
        const progress = Math.min(1, y / DISTANCE_THRESHOLD)
        skipOp = progress * 0.8
      }
    }

    if (likeOverlayRef.current) likeOverlayRef.current.style.opacity = String(likeOp)
    if (dislikeOverlayRef.current) dislikeOverlayRef.current.style.opacity = String(dislikeOp)
    if (skipOverlayRef.current) skipOverlayRef.current.style.opacity = String(skipOp)
  }, [])

  const getVelocity = useCallback((): { vx: number; vy: number } => {
    const history = touchHistoryRef.current
    if (history.length < 2) return { vx: 0, vy: 0 }

    const latest = history[history.length - 1]
    const cutoff = latest.time - 100
    let oldest = history[0]
    for (const p of history) {
      if (p.time >= cutoff) {
        oldest = p
        break
      }
    }

    const dt = latest.time - oldest.time
    if (dt === 0) return { vx: 0, vy: 0 }

    return {
      vx: (latest.x - oldest.x) / dt,
      vy: (latest.y - oldest.y) / dt
    }
  }, [])

  const clearOverlayTransitions = useCallback(() => {
    for (const ref of [likeOverlayRef, dislikeOverlayRef, skipOverlayRef]) {
      if (ref.current) ref.current.style.transition = ''
    }
  }, [])

  const resetOverlays = useCallback(
    (duration: number, easing: string) => {
      for (const ref of [likeOverlayRef, dislikeOverlayRef, skipOverlayRef]) {
        if (ref.current) {
          ref.current.style.transition = `opacity ${duration}ms ${easing}`
          ref.current.style.opacity = '0'
        }
      }
    },
    []
  )

  const springBack = useCallback(() => {
    const card = cardRef.current
    if (!card) return

    const { x, y } = offsetRef.current
    const distance = Math.sqrt(x * x + y * y)
    const duration = Math.max(100, Math.min(300, distance * 1.5))
    const easing = 'cubic-bezier(0.25, 0.46, 0.45, 0.94)'

    card.style.transition = `transform ${duration}ms ${easing}`
    card.style.transform = 'translate(0px, 0px) rotate(0deg)'
    resetOverlays(duration, easing)

    let done = false
    const cleanup = () => {
      if (done) return
      done = true
      if (exitTimeoutRef.current) clearTimeout(exitTimeoutRef.current)
      card.style.transition = ''
      clearOverlayTransitions()
      offsetRef.current = { x: 0, y: 0 }
      setSwipeState('idle')
    }

    const handler = (e: TransitionEvent) => {
      if (e.propertyName !== 'transform') return
      card.removeEventListener('transitionend', handler)
      cleanup()
    }
    card.addEventListener('transitionend', handler)
    exitTimeoutRef.current = setTimeout(cleanup, duration + 50)
  }, [resetOverlays, clearOverlayTransitions])

  const exitAndCallback = useCallback(
    (
      direction: 'like' | 'dislike' | 'skip',
      currentOffset: { x: number; y: number },
      velocity: { vx: number; vy: number }
    ) => {
      const card = cardRef.current
      if (!card) return

      setSwipeState('exiting')

      let targetX: number
      let targetY: number

      if (direction === 'like') {
        targetX = window.innerWidth * 1.5
        targetY = currentOffset.y
      } else if (direction === 'dislike') {
        targetX = -window.innerWidth * 1.5
        targetY = currentOffset.y
      } else {
        targetX = currentOffset.x
        targetY = window.innerHeight * 1.5
      }

      const dx = targetX - currentOffset.x
      const dy = targetY - currentOffset.y
      const remainingDist = Math.sqrt(dx * dx + dy * dy)
      const speed = Math.max(Math.abs(direction === 'skip' ? velocity.vy : velocity.vx), 1.0)
      const duration = Math.max(150, Math.min(400, remainingDist / speed))

      // Lock active overlay to full opacity
      const activeRef =
        direction === 'like'
          ? likeOverlayRef
          : direction === 'dislike'
            ? dislikeOverlayRef
            : skipOverlayRef
      if (activeRef.current) activeRef.current.style.opacity = '1'

      card.style.transition = `transform ${duration}ms ease-in`
      card.style.transform = `translate(${targetX}px, ${targetY}px) rotate(${targetX * 0.08}deg)`

      const cb = direction === 'like' ? onLike : direction === 'dislike' ? onDislike : onSkip
      let called = false

      const onComplete = () => {
        if (called) return
        called = true
        if (exitTimeoutRef.current) clearTimeout(exitTimeoutRef.current)
        cb()
      }

      const handler = (e: TransitionEvent) => {
        if (e.propertyName !== 'transform') return
        card.removeEventListener('transitionend', handler)
        onComplete()
      }
      card.addEventListener('transitionend', handler)
      exitTimeoutRef.current = setTimeout(onComplete, duration + 100)
    },
    [onLike, onDislike, onSkip]
  )

  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      if (disabled || swipeStateRef.current === 'exiting') return
      if (e.touches.length > 1) return

      const touch = e.touches[0]
      touchStartRef.current = { x: touch.clientX, y: touch.clientY }
      offsetRef.current = { x: 0, y: 0 }
      touchHistoryRef.current = [{ x: touch.clientX, y: touch.clientY, time: performance.now() }]

      if (cardRef.current) cardRef.current.style.transition = ''

      setSwipeState('dragging')
    },
    [disabled]
  )

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!touchStartRef.current || disabled || swipeStateRef.current !== 'dragging') return
      if (e.touches.length > 1) return

      const touch = e.touches[0]
      const dx = touch.clientX - touchStartRef.current.x
      const dy = Math.max(0, touch.clientY - touchStartRef.current.y)
      offsetRef.current = { x: dx, y: dy }

      const now = performance.now()
      touchHistoryRef.current.push({ x: touch.clientX, y: touch.clientY, time: now })
      while (touchHistoryRef.current.length > 5) touchHistoryRef.current.shift()

      if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current)
      animFrameRef.current = requestAnimationFrame(() => {
        updateCardDOM(dx, dy)
        updateOverlayDOM(dx, dy)
      })
    },
    [disabled, updateCardDOM, updateOverlayDOM]
  )

  const handleTouchEnd = useCallback(() => {
    if (!touchStartRef.current || swipeStateRef.current !== 'dragging') return

    if (animFrameRef.current) {
      cancelAnimationFrame(animFrameRef.current)
      animFrameRef.current = 0
    }

    const { x, y } = offsetRef.current
    const absX = Math.abs(x)
    const velocity = getVelocity()

    // Horizontal action (like/dislike)
    if (absX > y) {
      const distanceMet = absX > DISTANCE_THRESHOLD
      const velocityMet = Math.abs(velocity.vx) > VELOCITY_THRESHOLD && absX > MIN_FLICK_DISTANCE
      if (distanceMet || velocityMet) {
        exitAndCallback(x > 0 ? 'like' : 'dislike', offsetRef.current, velocity)
        touchStartRef.current = null
        return
      }
    }

    // Vertical action (skip — downward only)
    if (y > absX) {
      const distanceMet = y > DISTANCE_THRESHOLD
      const velocityMet = velocity.vy > VELOCITY_THRESHOLD && y > MIN_FLICK_DISTANCE
      if (distanceMet || velocityMet) {
        exitAndCallback('skip', offsetRef.current, velocity)
        touchStartRef.current = null
        return
      }
    }

    springBack()
    touchStartRef.current = null
  }, [getVelocity, exitAndCallback, springBack])

  const handleTouchCancel = useCallback(() => {
    if (swipeStateRef.current === 'dragging') {
      if (animFrameRef.current) {
        cancelAnimationFrame(animFrameRef.current)
        animFrameRef.current = 0
      }
      springBack()
    }
    touchStartRef.current = null
  }, [springBack])

  const handleButtonAction = useCallback(
    (direction: 'like' | 'dislike' | 'skip') => {
      if (disabled || swipeStateRef.current === 'exiting') return
      exitAndCallback(direction, { x: 0, y: 0 }, { vx: 0, vy: 0 })
    },
    [disabled, exitAndCallback]
  )

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-200 overflow-hidden">
      <Card
        ref={cardRef}
        className="w-full max-w-md h-[80vh] flex flex-col bg-zinc-900 border-zinc-800 overflow-hidden shadow-2xl relative touch-none select-none"
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        onTouchCancel={handleTouchCancel}
      >
        {/* Swipe overlay indicators — always rendered */}
        <div
          ref={likeOverlayRef}
          className="absolute inset-0 z-30 pointer-events-none border-4 border-green-500 rounded-xl flex items-center justify-center"
          style={{ opacity: 0 }}
        >
          <ThumbsUp className="w-20 h-20 text-green-500 fill-current drop-shadow-lg" />
        </div>
        <div
          ref={dislikeOverlayRef}
          className="absolute inset-0 z-30 pointer-events-none border-4 border-red-500 rounded-xl flex items-center justify-center"
          style={{ opacity: 0 }}
        >
          <ThumbsDown className="w-20 h-20 text-red-500 drop-shadow-lg" />
        </div>
        <div
          ref={skipOverlayRef}
          className="absolute inset-0 z-30 pointer-events-none border-4 border-zinc-400 rounded-xl flex items-center justify-center"
          style={{ opacity: 0 }}
        >
          <SkipForward className="w-20 h-20 text-zinc-400 drop-shadow-lg" />
        </div>

        {/* Movie Poster Background */}
        <div className="absolute inset-0 z-0">
          <Image
            src={movie.poster_url || '/placeholder-movie.jpg'}
            alt={movie.title}
            fill
            sizes="(max-width: 768px) 100vw, 500px"
            className={`object-cover transition-opacity duration-500 ${isImageLoading ? 'opacity-0' : 'opacity-60'}`}
            priority
            unoptimized
            onLoad={() => setIsImageLoading(false)}
          />
          {isImageLoading && <Skeleton className="absolute inset-0 bg-zinc-800" />}
          <div className="absolute inset-0 bg-gradient-to-t from-zinc-950 via-zinc-950/60 to-transparent" />
        </div>

        {/* Content */}
        <CardContent className="relative z-10 flex-1 flex flex-col justify-end p-6 pb-24 space-y-4">
          <div>
            <h2 className="text-3xl font-bold text-white shadow-black drop-shadow-md leading-tight">
              {movie.title}
            </h2>
            <div className="flex flex-wrap gap-2 mt-2">
              {movie.release_date && (
                <Badge
                  variant="secondary"
                  className="bg-black/40 text-zinc-300 hover:bg-black/60 backdrop-blur-md border-0"
                >
                  <Calendar className="w-3 h-3 mr-1" />
                  {new Date(movie.release_date).getFullYear()}
                </Badge>
              )}
              {movie.rating && (
                <Badge
                  variant="secondary"
                  className="bg-black/40 text-yellow-400 hover:bg-black/60 backdrop-blur-md border-0"
                >
                  <Star className="w-3 h-3 mr-1 fill-yellow-400" />
                  {movie.rating}
                </Badge>
              )}
              {movie.runtime && (
                <Badge
                  variant="secondary"
                  className="bg-black/40 text-zinc-300 hover:bg-black/60 backdrop-blur-md border-0"
                >
                  <Clock className="w-3 h-3 mr-1" />
                  {movie.runtime} min
                </Badge>
              )}
            </div>
          </div>

          <p className="text-zinc-200 line-clamp-4 text-sm leading-relaxed drop-shadow-md">
            {movie.overview}
          </p>

          <div className="flex flex-wrap gap-2 pt-2 pb-6">
            {movie.genres.slice(0, 3).map((genre) => (
              <Badge
                key={genre}
                variant="outline"
                className="text-xs font-medium text-zinc-400 border-zinc-700/50 bg-black/40 backdrop-blur-md"
              >
                {genre}
              </Badge>
            ))}
          </div>
        </CardContent>

        {/* Action Buttons */}
        <div className="absolute bottom-6 left-0 right-0 px-8 flex justify-between items-center z-20 gap-4">
          <Button
            size="icon"
            variant="ghost"
            className="w-16 h-16 rounded-full text-red-400 bg-red-400/10 hover:bg-red-500/20 hover:text-red-500 backdrop-blur-md border border-white/10 transition-all active:scale-95"
            onClick={() => handleButtonAction('dislike')}
            disabled={disabled || swipeState === 'exiting'}
          >
            <ThumbsDown className="w-8 h-8" />
          </Button>

          <Button
            size="icon"
            variant="ghost"
            className="w-16 h-16 rounded-full text-zinc-400 bg-zinc-800/50 hover:bg-zinc-700/50 hover:text-white backdrop-blur-md border border-white/10 transition-all active:scale-95"
            onClick={() => handleButtonAction('skip')}
            disabled={disabled || swipeState === 'exiting'}
          >
            <SkipForward className="w-6 h-6" />
          </Button>

          <Button
            size="icon"
            variant="ghost"
            className="w-16 h-16 rounded-full bg-green-500/20 hover:bg-green-500/30 text-green-500 backdrop-blur-md border border-green-500/30 transition-all active:scale-95 shadow-lg shadow-green-500/10 hover:text-green-400"
            onClick={() => handleButtonAction('like')}
            disabled={disabled || swipeState === 'exiting'}
          >
            <ThumbsUp className="w-8 h-8 fill-current" />
          </Button>
        </div>
      </Card>
    </div>
  )
}
