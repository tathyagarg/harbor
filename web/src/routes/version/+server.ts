import { PUBLIC_VERSION } from '$env/static/public';

// Constant endpoint to help in install script.
export function GET() {
  return new Response(JSON.stringify({
    version: PUBLIC_VERSION
  }))
}
