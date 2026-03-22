package dev.kode.lsp

import org.eclipse.lsp4j.launch.LSPLauncher
import java.io.InputStream
import java.io.OutputStream

fun main() {
    val server = KodeLspServer()
    startServer(server, System.`in`, System.out)
}

fun startServer(server: KodeLspServer, input: InputStream, output: OutputStream) {
    val launcher = LSPLauncher.createServerLauncher(server, input, output)
    val client = launcher.remoteProxy
    server.connect(client)
    launcher.startListening()
}
