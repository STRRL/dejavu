/** @type {import('next').NextConfig} */
const nextConfig = {
    // output: 'export',
    async rewrites(){
        return {
            fallback:[
                {
                    source: '/api/:path*',
                    destination: 'http://localhost:12333/api/:path*'
                }
            ]
        }
    }
}

module.exports = nextConfig
