export type P2pPhase = "idle" | "starting" | "active" | "stopping";

export interface P2pPeer {
  id: string;
  route: "direct" | "relay" | null;
  rttMs: number | null;
}

export interface P2pStatus {
  phase: P2pPhase;
  mode: "host" | "join" | null;
  localAddress: string | null;
  shareUri: string | null;
  playerCount: number;
  hostPort: number | null;
  route: "direct" | "relay" | null;
  rttMs: number | null;
  txBytes: number;
  rxBytes: number;
  hostPeers: P2pPeer[];
  error: string | null;
  message: string | null;
}

export const emptyP2pStatus: P2pStatus = {
  phase: "idle",
  mode: null,
  localAddress: null,
  shareUri: null,
  playerCount: 0,
  hostPort: null,
  route: null,
  rttMs: null,
  txBytes: 0,
  rxBytes: 0,
  hostPeers: [],
  error: null,
  message: null,
};
