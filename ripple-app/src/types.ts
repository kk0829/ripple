export interface DeviceInfo {
  fingerprint: string;
  name: string;
  os_type: string;
  ip: string;
  port: number;
}

export interface ChatMessage {
  id: string;
  type: "text" | "file_ref" | "image" | "system" | "ack" | "typing";
  timestamp: number;
  payload: {
    content: string;
    format: "plain" | "markdown";
  };
  ref_id?: string;
}

export interface SendMessageRequest {
  target_ip: string;
  target_port: number;
  message: ChatMessage;
}
