/**
 * WebSocket hook for receiving events from daemon
 */

import { useEffect, useRef, useState } from 'react';

const WS_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:8871/events';

export interface WebSocketEvent {
  type: 'state_changed' | 'parsed' | 'raw_line';
  [key: string]: any;
}

export function useWebSocket(onEvent?: (event: WebSocketEvent) => void) {
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const connect = () => {
    try {
      const ws = new WebSocket(WS_URL);

      ws.onopen = () => {
        console.log('WebSocket connected');
        setIsConnected(true);
        setError(null);
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          onEvent?.(data);
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      ws.onerror = (event) => {
        console.error('WebSocket error:', event);
        setError('WebSocket connection error');
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setIsConnected(false);

        // Reconnect after 3 seconds
        reconnectTimeoutRef.current = setTimeout(() => {
          console.log('Reconnecting WebSocket...');
          connect();
        }, 3000);
      };

      wsRef.current = ws;
    } catch (e) {
      console.error('Failed to create WebSocket:', e);
      setError('Failed to connect to daemon');
    }
  };

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  return { isConnected, error };
}
