import fastify from 'fastify'

const server = fastify({ logger: true })

server.get('/', async (request, reply) => {
    return { hello: 'world' }
})

async function start() {
    await server.listen({ port: 3002 })
}
start()
