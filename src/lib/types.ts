export interface FadingStroke {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  color: string;
  size: number;
  startTime: number;
  duration: number;
}

export interface User {
  id: string;
  nickname: string;
  color: string;
  status: 'online' | 'busy';
}

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'error';
