export const useWS = () => {
    let socket: WebSocket | null = null;

    const connect = (ws_url: string) => {
        // WebSocket 接続を作成
        if (!socket) {
            socket = new WebSocket(ws_url);

            // // 接続が開いたときのイベント
            // socket.addEventListener("open", (event) => {
            //     if (socket) {
            //         socket.send("Hello Server!");
            //     }
            // });

            // メッセージの待ち受け
            // socket.addEventListener("message", (event) => {
            //     console.log("Message from server ", event.data);
            // });
        }

        return socket;
    }

    return {
        connect
    }
}
export default {
    useWS
}