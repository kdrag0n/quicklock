module.exports = function(app) {
  app.use((req, res, next) => {
    // Enable SharedArrayBuffer
    res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp')
    res.setHeader('Cross-Origin-Opener-Policy', 'same-origin')
    next()
  });
}
