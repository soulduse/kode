package dev.kode.lsp

import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.services.TextDocumentService
import java.util.concurrent.CompletableFuture

/**
 * Proxies standard textDocument/* requests to the inner kotlin-language-server,
 * augmenting with Spring-specific results where applicable.
 */
class KodeTextDocumentService(private val server: KodeLspServer) : TextDocumentService {

    override fun completion(params: CompletionParams): CompletableFuture<Either<List<CompletionItem>, CompletionList>> {
        val proxy = server.getProxy()

        // If proxy is available, forward to fwcd server
        val proxyResult = proxy?.completion(params)

        // TODO: augment with Spring-specific completions for application.yml
        return proxyResult ?: CompletableFuture.completedFuture(
            Either.forLeft(emptyList())
        )
    }

    override fun hover(params: HoverParams): CompletableFuture<Hover?> {
        return server.getProxy()?.hover(params)
            ?: CompletableFuture.completedFuture(null)
    }

    override fun definition(params: DefinitionParams): CompletableFuture<Either<List<out Location>, List<out LocationLink>>> {
        return server.getProxy()?.definition(params)
            ?: CompletableFuture.completedFuture(Either.forLeft(emptyList()))
    }

    override fun references(params: ReferenceParams): CompletableFuture<List<out Location>> {
        return server.getProxy()?.references(params)
            ?: CompletableFuture.completedFuture(emptyList())
    }

    override fun documentSymbol(params: DocumentSymbolParams): CompletableFuture<List<Either<SymbolInformation, DocumentSymbol>>> {
        return server.getProxy()?.documentSymbol(params)
            ?: CompletableFuture.completedFuture(emptyList())
    }

    override fun codeAction(params: CodeActionParams): CompletableFuture<List<Either<Command, CodeAction>>> {
        return server.getProxy()?.codeAction(params)
            ?: CompletableFuture.completedFuture(emptyList())
    }

    override fun didOpen(params: DidOpenTextDocumentParams) {
        server.getProxy()?.didOpen(params)
    }

    override fun didChange(params: DidChangeTextDocumentParams) {
        server.getProxy()?.didChange(params)
    }

    override fun didClose(params: DidCloseTextDocumentParams) {
        server.getProxy()?.didClose(params)
    }

    override fun didSave(params: DidSaveTextDocumentParams) {
        server.getProxy()?.didSave(params)

        // Re-index on save for Spring annotations
        CompletableFuture.runAsync {
            server.springIndexer.reindexFile(params.textDocument.uri)
        }
    }
}
