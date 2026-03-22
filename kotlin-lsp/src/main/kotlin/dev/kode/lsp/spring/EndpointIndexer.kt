package dev.kode.lsp.spring

import java.util.concurrent.CopyOnWriteArrayList

/**
 * Stores discovered REST endpoints.
 */
class EndpointIndexer {

    private val endpoints = CopyOnWriteArrayList<RestEndpoint>()

    fun add(endpoint: RestEndpoint) {
        endpoints.add(endpoint)
    }

    fun getAll(): List<RestEndpoint> = endpoints.toList()

    fun removeByFile(fileUri: String) {
        endpoints.removeIf { it.fileUri == fileUri }
    }

    fun clear() {
        endpoints.clear()
    }
}

/**
 * A discovered REST endpoint.
 */
data class RestEndpoint(
    val method: String,
    val path: String,
    val handlerClass: String,
    val handlerMethod: String,
    val fileUri: String,
    val line: Int,
    val character: Int,
    val parameters: List<EndpointParam>,
)

/**
 * A parameter of a REST endpoint.
 */
data class EndpointParam(
    val name: String,
    val paramType: String,
    val source: ParamSource,
)

enum class ParamSource {
    PATH, QUERY, BODY, HEADER,
}
