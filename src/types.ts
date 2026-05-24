export interface ImageTarget {
  id: string
  name: string
  thumbnail: string
  matchThreshold: number
  lastMatch: {
    x: number
    y: number
    confidence: number
  } | null
}

export interface WorkflowNode {
  id: string
  type: string
  label: string
  x: number
  y: number
  config: Record<string, unknown>
}

export interface WorkflowEdge {
  source: string
  target: string
  label?: string
}

export interface ExecutionLog {
  id: string
  timestamp: Date
  level: 'info' | 'success' | 'warning' | 'error'
  message: string
  nodeId: string | null
}

export interface Screenshot {
  id: string
  timestamp: Date
  image: string
  label: string
}

export interface DetectedRegion {
  id: string
  x: number
  y: number
  width: number
  height: number
  confidence: number
  targetName: string
}

export interface ActionBlock {
  type: string
  label: string
  icon: string
  color: string
  description: string
}
