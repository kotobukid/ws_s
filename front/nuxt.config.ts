import dns from "dns";

// IPv4優先
dns.setDefaultResultOrder("ipv4first");

// コマンドライン引数の解析
const getCommandLineArg = (key: string): string | undefined => {
  const index = process.argv.indexOf(`--${key}`);
  if (index !== -1 && process.argv.length > index + 1) {
    return process.argv[index + 1];
  }
  return undefined;
};

// コマンドライン引数 "--hostname" を取得
const cli_hostname = getCommandLineArg("hostname");

// 環境変数と組み合わせてホスト名を設定
const env_hostname: string = process.env.HOSTNAME || "127.0.0.1:8080";
const final_hostname: string = cli_hostname || env_hostname;

const hostname = `ws://${final_hostname}/ws`;

export default defineNuxtConfig({
  ssr: false,
  runtimeConfig: {
    public: {
      wsHost: hostname
    }
  },
  app: {
    head: {
      title: "test app",
      htmlAttrs: {lang: 'ja'},
      meta: [
        {charset: 'utf-8'},
        {name: 'viewport', content: 'width=device-width, initial-scale=1'}
      ]
    }
  },
  vite: {
    server: {
      proxy: {
        '/api': {
          target: 'http://127.0.0.1:8080',
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/api/, '/api'),
          ws: true // WebSocket support
        },
        '/ws': { // WebSocket proxy for 127.0.0.1:8080
          target: 'ws://127.0.0.1:8080',
          changeOrigin: true,
          ws: true
        }
      }
    }
  },
  generate: {
    routes: [
      "/",
      "/additional/"
    ]
  },
  modules: ["@pinia/nuxt"],
  compatibilityDate: '2024-12-03'
})