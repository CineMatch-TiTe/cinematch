import type { NextConfig } from 'next'

const nextConfig: NextConfig = {
  /* config options here */
  reactCompiler: true,
  output: 'standalone',
  images: {
    remotePatterns: [new URL('https://image.tmdb.org/**')]
  }
}

export default nextConfig
