import init, {
    convert_to_bytes,
    serialize_text_message,
    deserialize_text_message,
    serialize_list_message,
    MessageType
} from "~/public/pkg/message_pack_wasm"

export const useWasmTest = () => {
    const test = async () => {
        await init();

        // Enumも使用可能
        console.log(MessageType)

        const serialized: Uint8Array = serialize_text_message("jiro", 42, convert_to_bytes(MessageType.Chat), "hello");
        console.log(serialized);

        const deserialized = deserialize_text_message(serialized);
        console.log(deserialized);
    }

    const create_text_message = (sender: string, room: number, content: string): Uint8Array => {
        return serialize_text_message(sender, room, convert_to_bytes(MessageType.Chat), content);
    }

    const create_list_message = (sender: string, room: number, target: string | undefined): Uint8Array => {
        return serialize_list_message(sender, room, target);
    }

    return {
        init,
        test,
        create_text_message,
        deserialize_text_message,

        create_list_message,
    }
}

