'use client'

import { useEffect, useRef, useState, useCallback } from 'react'
import { ServerMessage, ClientMessage } from '@/lib/ws-types'
import { useAuth } from '@/lib/auth-context'

interface UsePartySocketOptions {
  partyId?: string
  onMessage?: (message: ServerMessage) => void
  onConnect?: () => void
  onDisconnect?: () => void
}

export function usePartySocket({ partyId, onMessage, onConnect, onDisconnect }: UsePartySocketOptions = {}) {
  const { getToken } = useAuth()
  const token = getToken()
  const socketRef = useRef<WebSocket | null>(null)
  const [isConnected, setIsConnected] = useState(false)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const reconnectAttemptsRef = useRef(0)

  const connectRef = useRef<() => void>(() => {})

  // Store callback props in refs so `connect` doesn't depend on their identity
  const onMessageRef = useRef(onMessage)
  const onConnectRef = useRef(onConnect)
  const onDisconnectRef = useRef(onDisconnect)

  useEffect(() => { onMessageRef.current = onMessage }, [onMessage])
  useEffect(() => { onConnectRef.current = onConnect }, [onConnect])
  useEffect(() => { onDisconnectRef.current = onDisconnect }, [onDisconnect])

  const connect = useCallback(() => {
    // If we're already connected or connecting, or have no token, don't connect
    if (socketRef.current?.readyState === WebSocket.OPEN ||
        socketRef.current?.readyState === WebSocket.CONNECTING ||
        !token) {
      return
    }

    const baseUrl = process.env.NEXT_PUBLIC_API_BASE || 'https://api.cinematch.space'
    const wsBase = baseUrl.replace(/^http/, 'ws')

    // Add token as query param since websockets don't support custom headers in browser
    const wsUrl = `${wsBase}/api/ws?token=${encodeURIComponent(token)}`

    console.log('[WebSocket] Connecting...')
    const ws = new WebSocket(wsUrl)
    socketRef.current = ws

    ws.onopen = () => {
      console.log('[WebSocket] Connected')
      setIsConnected(true)
      reconnectAttemptsRef.current = 0 // Reset attempts on successful connection
      onConnectRef.current?.()
    }

    ws.onmessage = async (event) => {
      // Handle heartbeats (binary payload of timestamp)
      if (event.data instanceof Blob) {
        // Echo back the ping payload as a pong
        ws.send(event.data)
        return
      }

      try {
        const message: ServerMessage = JSON.parse(event.data)
        onMessageRef.current?.(message)
      } catch (e) {
        console.error('[WebSocket] Failed to parse message', event.data, e)
      }
    }

    ws.onclose = (event) => {
      console.log('[WebSocket] Disconnected', event.code, event.reason)
      setIsConnected(false)
      socketRef.current = null
      onDisconnectRef.current?.()

      // Attempt reconnection with exponential backoff (max 8 seconds delay)
      const attempts = reconnectAttemptsRef.current
      if (attempts < 10) {
        reconnectAttemptsRef.current += 1
        const delay = Math.min(1000 * Math.pow(2, attempts), 8000)
        console.log(`[WebSocket] Reconnecting in ${delay}ms...`)

        if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
        reconnectTimeoutRef.current = setTimeout(() => connectRef.current(), delay)
      }
    }

    ws.onerror = (error) => {
      console.error('[WebSocket] Error', error)
      // Error will typically be followed by onclose where we handle reconnection
    }
  }, [token])

  // Keep connectRef fresh
  useEffect(() => {
    connectRef.current = connect
  }, [connect])

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }
    
    if (socketRef.current) {
      socketRef.current.close()
      socketRef.current = null
    }
  }, [])

  const sendMessage = useCallback((msg: ClientMessage) => {
    if (socketRef.current?.readyState === WebSocket.OPEN) {
      socketRef.current.send(JSON.stringify(msg))
    } else {
      console.warn('[WebSocket] Cannot send message, socket not open', msg)
    }
  }, [])

  // Manage connection lifecycle
  useEffect(() => {
    if (partyId && token) {
      connect()
    }
    return () => {
      disconnect()
    }
  }, [partyId, token, connect, disconnect])

  return {
    isConnected,
    sendMessage,
    disconnect,
    reconnect: connect
  }
}
