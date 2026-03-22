package dev.kode.lsp

import org.eclipse.lsp4j.DidChangeConfigurationParams
import org.eclipse.lsp4j.DidChangeWatchedFilesParams
import org.eclipse.lsp4j.services.WorkspaceService

class KodeWorkspaceService : WorkspaceService {

    override fun didChangeConfiguration(params: DidChangeConfigurationParams) {
        // No-op for now
    }

    override fun didChangeWatchedFiles(params: DidChangeWatchedFilesParams) {
        // No-op for now
    }
}
