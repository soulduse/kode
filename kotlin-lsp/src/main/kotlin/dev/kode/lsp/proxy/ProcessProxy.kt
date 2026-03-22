package dev.kode.lsp.proxy

import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.launch.LSPLauncher
import org.eclipse.lsp4j.services.LanguageServer
import java.util.concurrent.CompletableFuture

/**
 * Proxies LSP requests to an external kotlin-language-server process.
 */
class ProcessProxy private constructor(
    private val process: Process,
    private val server: LanguageServer,
) {

    companion object {
        /**
         * Start a kotlin-language-server process and connect via stdio.
         */
        fun start(command: String, vararg args: String): ProcessProxy {
            val cmdList = mutableListOf(command).apply { addAll(args) }
            val process = ProcessBuilder(cmdList)
                .redirectError(ProcessBuilder.Redirect.INHERIT)
                .start()

            val launcher = LSPLauncher.createClientLauncher(
                NoOpLanguageClient(),
                process.inputStream,
                process.outputStream,
            )
            launcher.startListening()

            return ProcessProxy(process, launcher.remoteProxy)
        }
    }

    fun sendInitialize(params: InitializeParams): InitializeResult? {
        return try {
            server.initialize(params).get()
        } catch (e: Exception) {
            null
        }
    }

    fun completion(params: CompletionParams): CompletableFuture<Either<List<CompletionItem>, CompletionList>> {
        return server.textDocumentService.completion(params)
    }

    fun hover(params: HoverParams): CompletableFuture<Hover?> {
        return server.textDocumentService.hover(params)
    }

    fun definition(params: DefinitionParams): CompletableFuture<Either<List<out Location>, List<out LocationLink>>> {
        return server.textDocumentService.definition(params)
    }

    fun references(params: ReferenceParams): CompletableFuture<List<out Location>> {
        return server.textDocumentService.references(params)
    }

    fun documentSymbol(params: DocumentSymbolParams): CompletableFuture<List<Either<SymbolInformation, DocumentSymbol>>> {
        return server.textDocumentService.documentSymbol(params)
    }

    fun codeAction(params: CodeActionParams): CompletableFuture<List<Either<Command, CodeAction>>> {
        return server.textDocumentService.codeAction(params)
    }

    fun didOpen(params: DidOpenTextDocumentParams) {
        server.textDocumentService.didOpen(params)
    }

    fun didChange(params: DidChangeTextDocumentParams) {
        server.textDocumentService.didChange(params)
    }

    fun didClose(params: DidCloseTextDocumentParams) {
        server.textDocumentService.didClose(params)
    }

    fun didSave(params: DidSaveTextDocumentParams) {
        server.textDocumentService.didSave(params)
    }

    fun shutdown() {
        try {
            server.shutdown().get()
        } catch (_: Exception) {}
    }

    fun exit() {
        try {
            server.exit()
        } catch (_: Exception) {}
        process.destroyForcibly()
    }
}

/**
 * No-op language client for the proxy connection.
 */
private class NoOpLanguageClient : org.eclipse.lsp4j.services.LanguageClient {
    override fun telemetryEvent(obj: Any?) {}
    override fun publishDiagnostics(diagnostics: PublishDiagnosticsParams?) {}
    override fun showMessage(messageParams: MessageParams?) {}
    override fun showMessageRequest(requestParams: ShowMessageRequestParams?): CompletableFuture<MessageActionItem> {
        return CompletableFuture.completedFuture(null)
    }
    override fun logMessage(message: MessageParams?) {}
}
