import init, {
    serialize_exit_message,
    serialize_exit_message_string,
    serialize_text_message,
    deserialize_text_message
} from "~/public/pkg/message_pack_wasm"

export const useWasmTest = () => {
    const run = async () => {
        await init();
        console.log(serialize_text_message.toString())
        console.log(deserialize_text_message.toString())

        const serialized: Uint8Array = serialize_text_message("kotobukid", 42, 0x01, "hello");
        console.log(serialized);
        const deserialized = deserialize_text_message(serialized);
        console.log(deserialized);
    }

    return {
        run
    }
}

