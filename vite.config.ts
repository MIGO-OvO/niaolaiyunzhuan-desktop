import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { spawn } from 'node:child_process';
import path from 'node:path';

export default defineConfig({
  plugins: [react(), {
    name: 'rust-preview',
    configureServer(server) {
      server.middlewares.use('/api', async (request, response) => {
        response.setHeader('Content-Type', 'application/json; charset=utf-8');
        if (request.method !== 'POST') { response.statusCode = 405; response.end('只接受 POST'); return; }
        let body = '';
        try {
          for await (const chunk of request) {
            body += chunk.toString();
            if (body.length > 24 * 1024 * 1024) throw new Error('图片超过大小限制');
          }
          const command = request.url?.slice(1).split('?')[0];
          const args: unknown = JSON.parse(body);
          const binary = path.resolve('target/debug', process.platform === 'win32' ? 'preview-bridge.exe' : 'preview-bridge');
          const child = spawn(binary, [path.resolve(process.env.NIAOLAI_PREVIEW_DIR ?? '.local/preview')], { windowsHide: true });
          let output = '';
          child.stdout.on('data', chunk => { output += chunk.toString(); });
          child.once('error', error => { response.statusCode = 500; response.end(error.message); });
          child.once('close', code => {
            if (response.writableEnded) return;
            try {
              if (code !== 0) throw new Error('预览核心未能运行，请重新执行 npm run dev');
              const result = JSON.parse(output);
              response.statusCode = result.ok ? 200 : 400;
              response.end(result.ok ? JSON.stringify(result.value) : result.error);
            } catch (error) { response.statusCode = 500; response.end(String(error)); }
          });
          child.stdin.end(JSON.stringify({ command, args }));
        } catch (error) { response.statusCode = 400; response.end(String(error)); }
      });
    },
  }],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: 'es2022' },
});
