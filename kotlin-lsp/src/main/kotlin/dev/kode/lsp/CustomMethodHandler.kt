package dev.kode.lsp

import com.google.gson.Gson
import com.google.gson.JsonElement

/**
 * Handles custom spring/* JSON-RPC methods.
 */
class CustomMethodHandler(private val server: KodeLspServer) {

    private val gson = Gson()

    /**
     * Dispatch a custom method request.
     * Returns JSON result or null if method is unknown.
     */
    fun handle(method: String, params: JsonElement?): JsonElement? {
        return when (method) {
            "spring/beans" -> handleBeans()
            "spring/endpoints" -> handleEndpoints()
            "spring/beanGraph" -> handleBeanGraph()
            "spring/gradleTasks" -> handleGradleTasks()
            "spring/runTask" -> handleRunTask(params)
            "spring/configKeys" -> handleConfigKeys(params)
            else -> null
        }
    }

    private fun handleBeans(): JsonElement {
        val beans = server.springIndexer.getBeans()
        return gson.toJsonTree(beans)
    }

    private fun handleEndpoints(): JsonElement {
        val endpoints = server.springIndexer.getEndpoints()
        return gson.toJsonTree(endpoints)
    }

    private fun handleBeanGraph(): JsonElement {
        val graph = server.springIndexer.getBeanGraph()
        return gson.toJsonTree(graph)
    }

    private fun handleGradleTasks(): JsonElement {
        // TODO: Implement via GradleConnector
        return gson.toJsonTree(emptyList<Any>())
    }

    private fun handleRunTask(params: JsonElement?): JsonElement? {
        // TODO: Implement via TaskRunner
        return null
    }

    private fun handleConfigKeys(params: JsonElement?): JsonElement {
        // TODO: Implement via YamlCompleter
        return gson.toJsonTree(emptyList<Any>())
    }
}
