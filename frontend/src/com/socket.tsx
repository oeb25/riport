import * as React from "react";
import { Server2Client } from "./s2c";
import { Client2Server } from "./c2s";

export type WebSocketStatus =
  | {
      type: "INITIAL";
    }
  | {
      type: "CONNECTING";
      ws: WebSocket;
    }
  | {
      type: "CONNECTED";
      ws: WebSocket;
    }
  | {
      type: "CLOSED";
      ws: WebSocket;
    }
  | {
      type: "ERROR";
      ws: WebSocket | null;
    };

export const socket = (
  handler: (msg: Server2Client) => any
): [WebSocketStatus, (msg: Client2Server) => any] => {
  const [wsStatus, setWsStatus] = React.useState<WebSocketStatus>({
    type: "INITIAL"
  });

  React.useEffect(() => {
    const ws = wsStatus.type != "INITIAL" ? wsStatus.ws : null;
    if (ws) {
      // console.log("binding...");
      ws.onopen = () => {
        setWsStatus({ type: "CONNECTED", ws });
      };
      ws.onmessage = e => {
        const msg = JSON.parse(e.data);
        // console.log("REVICE", e.data);
        // setMessages([...messages, e.data]);
        // dispatch({ type: "NEW_MESSAGE", contents: e.data });
        handler(msg);
      };
      ws.onclose = e => {
        console.log("close", e);
        setWsStatus({ type: "CLOSED", ws });
      };
      ws.onerror = e => {
        console.log("error", e);
        setWsStatus({ type: "ERROR", ws });
      };
    }
    if (
      wsStatus.type == "CLOSED" ||
      wsStatus.type == "ERROR" ||
      wsStatus.type == "INITIAL"
    ) {
      try {
        const ws = new WebSocket("ws://0.0.0.0:8000/ws/");
        setWsStatus({ type: "CONNECTING", ws });
      } catch (e) {
        setWsStatus({ type: "ERROR", ws });
      }
    }
  }, [wsStatus.type]);

  const ws = "ws" in wsStatus ? wsStatus.ws : null;

  React.useEffect(() => {
    const interval = setInterval(() => {
      // TODO: Maybe hearthbeat
      // console.log("This will run every second!", ws);
      // if (ws) {
      //   ws.send("Hello");
      // }
    }, 1000);
    return () => clearInterval(interval);
  }, [ws]);

  const send = (msg: Client2Server) => {
    const ws = "ws" in wsStatus ? wsStatus.ws : null;
    if (ws) {
      ws.send(JSON.stringify(msg));
    } else {
      console.error("DIDN'T SEND!");
    }
  };

  return [wsStatus, send];
};

export type Send = (msg: Client2Server) => void;

export const SocketContext = React.createContext<null | WebSocketStatus>(null);

export const SocketProvider: React.SFC<{ wsStatus: WebSocketStatus }> = ({
  children,
  wsStatus
}) => {
  return (
    <SocketContext.Provider value={wsStatus}>{children}</SocketContext.Provider>
  );
};
