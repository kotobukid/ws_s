import dns from "dns";

dns.setDefaultResultOrder("ipv4first");

export default defineNuxtConfig({
  ssr: false,
  runtimeConfig: {
    public: {
      wsHost: process.env.WS_HOST || "ws://127.0.0.1:8080/ws"
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
  modules: ["@pinia/nuxt"],
  compatibilityDate: '2024-12-03'
})