package dev.kode.lsp

import dev.kode.lsp.proxy.ProcessProxy
import dev.kode.lsp.spring.SpringIndexer
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.services.*
import java.util.concurrent.CompletableFuture

/**
 * Kode LSP Server — wraps fwcd/kotlin-language-server as a sidecar proxy
 * and adds Spring-specific custom method handling.
 */
class KodeLspServer : LanguageServer, LanguageClientAware {

    private var client: LanguageClient? = null
    private var proxy: ProcessProxy? = null
    private val textDocumentService = KodeTextDocumentService(this)
    private val workspaceService = KodeWorkspaceService()
    private val customMethodHandler = CustomMethodHandler(this)
    val springIndexer = SpringIndexer()

    override fun initialize(params: InitializeParams): CompletableFuture<InitializeResult> {
        val rootUri = params.rootUri

        // Try to start fwcd/kotlin-language-server as sidecar
        try {
            proxy = ProcessProxy.start("kotlin-language-server")
            proxy?.sendInitialize(params)
        } catch (e: Exception) {
            log("kotlin-language-server not available: ${e.message}")
        }

        // Trigger Spring indexing if root URI is available
        if (rootUri != null) {
            CompletableFuture.runAsync {
                springIndexer.indexProject(rootUri)
            }
        }

        val capabilities = ServerCapabilities().apply {
            textDocumentSync = Either.forLeft(TextDocumentSyncKind.Full)
            completionProvider = CompletionOptions(false, listOf(".", ":", "("))
            hoverProvider = Either.forLeft(true)
            definitionProvider = Either.forLeft(true)
            referencesProvider = Either.forLeft(true)
            documentSymbolProvider = Either.forLeft(true)
            codeActionProvider = Either.forLeft(true)
        }

        return CompletableFuture.completedFuture(
            InitializeResult(capabilities)
        )
    }

    override fun shutdown(): CompletableFuture<Any> {
        proxy?.shutdown()
        return CompletableFuture.completedFuture(null)
    }

    override fun exit() {
        proxy?.exit()
    }

    override fun getTextDocumentService(): TextDocumentService = textDocumentService

    override fun getWorkspaceService(): WorkspaceService = workspaceService

    override fun connect(client: LanguageClient) {
        this.client = client
    }

    fun getClient(): LanguageClient? = client

    fun getProxy(): ProcessProxy? = proxy

    fun getCustomMethodHandler(): CustomMethodHandler = customMethodHandler

    fun log(message: String) {
        client?.logMessage(MessageParams(MessageType.Info, message))
            ?: System.err.println("[kode-lsp] $message")
    }
}
