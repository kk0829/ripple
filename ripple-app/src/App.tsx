import { useEffect, useState, useRef, useCallback } from "react";
import {
  getDeviceList,
  sendMessage,
  onDeviceDiscovered,
  onDeviceRemoved,
} from "./api";
import type { DeviceInfo, ChatMessage } from "./types";
import "./App.css";

interface ChatEntry {
  msg: ChatMessage;
  direction: "sent" | "received";
}

function App() {
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<DeviceInfo | null>(null);
  const [chatHistory, setChatHistory] = useState<Record<string, ChatEntry[]>>({});
  const [inputText, setInputText] = useState("");
  const [sending, setSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    getDeviceList().then(setDevices).catch(console.error);

    const unlistenDiscovered = onDeviceDiscovered((device) => {
      setDevices((prev) => {
        if (prev.find((d) => d.fingerprint === device.fingerprint)) return prev;
        return [...prev, device];
      });
    });

    const unlistenRemoved = onDeviceRemoved((fingerprint) => {
      setDevices((prev) => prev.filter((d) => d.fingerprint !== fingerprint));
      setSelectedDevice((cur) =>
        cur?.fingerprint === fingerprint ? null : cur
      );
    });

    return () => {
      unlistenDiscovered.then((fn) => fn());
      unlistenRemoved.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [chatHistory, selectedDevice]);

  const handleSend = useCallback(async () => {
    if (!selectedDevice || !inputText.trim() || sending) return;

    const content = inputText.trim();
    setInputText("");
    setSending(true);

    const tempMsg: ChatMessage = {
      id: crypto.randomUUID(),
      type: "text",
      timestamp: Math.floor(Date.now() / 1000),
      payload: { content, format: "plain" },
    };

    const fp = selectedDevice.fingerprint;
    setChatHistory((prev) => ({
      ...prev,
      [fp]: [...(prev[fp] || []), { msg: tempMsg, direction: "sent" }],
    }));

    try {
      await sendMessage(selectedDevice.ip, selectedDevice.port, content);
    } catch (err) {
      console.error("Send failed:", err);
    } finally {
      setSending(false);
    }
  }, [selectedDevice, inputText, sending]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const currentChat = selectedDevice
    ? chatHistory[selectedDevice.fingerprint] || []
    : [];

  return (
    <div className="app">
      <div className="sidebar">
        <div className="sidebar-header">
          <div className="logo">R</div>
          <h1>Ripple</h1>
        </div>
        <div className="device-list">
          {devices.length === 0 ? (
            <div className="empty-state">
              <div className="icon">📡</div>
              <div>Searching for devices...</div>
              <div style={{ fontSize: 12 }}>
                Make sure other devices are on the same network
              </div>
            </div>
          ) : (
            devices.map((device) => (
              <div
                key={device.fingerprint}
                className={`device-item ${
                  selectedDevice?.fingerprint === device.fingerprint
                    ? "active"
                    : ""
                }`}
                onClick={() => setSelectedDevice(device)}
              >
                <div className="device-avatar">
                  {device.name.charAt(0).toUpperCase()}
                  <div className="online-dot" />
                </div>
                <div className="device-info">
                  <div className="device-name">{device.name}</div>
                  <div className="device-meta">
                    {device.os_type} · {device.ip}
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {selectedDevice ? (
        <div className="chat-area">
          <div className="chat-header">
            <div
              className="device-avatar"
              style={{ width: 32, height: 32, fontSize: 13 }}
            >
              {selectedDevice.name.charAt(0).toUpperCase()}
              <div className="online-dot" />
            </div>
            <div>
              <div className="name">{selectedDevice.name}</div>
              <div className="status">
                {selectedDevice.os_type} · {selectedDevice.ip}:
                {selectedDevice.port}
              </div>
            </div>
          </div>

          <div className="chat-messages">
            {currentChat.length === 0 ? (
              <div className="empty-state" style={{ height: "auto", flex: 1 }}>
                <div>No messages yet. Say hello!</div>
              </div>
            ) : (
              currentChat.map((entry) => (
                <div
                  key={entry.msg.id}
                  className={`message-bubble ${entry.direction}`}
                >
                  <div>{entry.msg.payload.content}</div>
                  <div className="message-time">
                    {new Date(entry.msg.timestamp * 1000).toLocaleTimeString()}
                  </div>
                </div>
              ))
            )}
            <div ref={messagesEndRef} />
          </div>

          <div className="chat-input-area">
            <textarea
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type a message..."
              rows={1}
            />
            <button onClick={handleSend} disabled={!inputText.trim() || sending}>
              Send
            </button>
          </div>
        </div>
      ) : (
        <div className="no-chat-selected">
          <div className="icon">💬</div>
          <div>Select a device to start chatting</div>
        </div>
      )}
    </div>
  );
}

export default App;
