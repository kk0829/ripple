import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { DeviceInfo, ChatMessage, SendMessageRequest } from "./types";

export async function getDeviceList(): Promise<DeviceInfo[]> {
  return invoke<DeviceInfo[]>("get_device_list");
}

export async function sendMessage(
  targetIp: string,
  targetPort: number,
  content: string,
  format: "plain" | "markdown" = "plain"
): Promise<string> {
  const message: ChatMessage = {
    id: crypto.randomUUID(),
    type: "text",
    timestamp: Math.floor(Date.now() / 1000),
    payload: { content, format },
  };

  const req: SendMessageRequest = {
    target_ip: targetIp,
    target_port: targetPort,
    message,
  };

  return invoke<string>("send_message", { req });
}

export function onDeviceDiscovered(
  handler: (device: DeviceInfo) => void
) {
  return listen<DeviceInfo>("device-discovered", (event) => {
    handler(event.payload);
  });
}

export function onDeviceRemoved(handler: (fingerprint: string) => void) {
  return listen<string>("device-removed", (event) => {
    handler(event.payload);
  });
}
