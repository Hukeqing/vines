import type { NextConfig } from "next";

const nextConfig: NextConfig = {
    async rewrites() {
        return [
            {
                source: '/api/:path*',
                destination: 'http://localhost:8080/api/:path*'  // 代理到本地后端
            }
        ]
    },
    reactStrictMode: true
};

export default nextConfig;
