package dev.kode.lsp.spring

import java.io.File
import java.net.URI
import java.util.concurrent.ConcurrentHashMap

/**
 * Scans Kotlin source files for Spring annotations and builds
 * a registry of beans and REST endpoints.
 */
class SpringIndexer {

    private val beanRegistry = BeanRegistry()
    private val endpointIndexer = EndpointIndexer()
    private val indexedFiles = ConcurrentHashMap<String, Long>()

    /**
     * Index an entire project from the root URI.
     */
    fun indexProject(rootUri: String) {
        val rootDir = uriToFile(rootUri) ?: return
        val kotlinFiles = rootDir.walk()
            .filter { it.extension == "kt" }
            .filter { !it.path.contains("/build/") && !it.path.contains("/test/") }
            .toList()

        for (file in kotlinFiles) {
            indexFile(file)
        }
    }

    /**
     * Re-index a single file (called on save).
     */
    fun reindexFile(fileUri: String) {
        val file = uriToFile(fileUri) ?: return
        if (file.extension != "kt") return

        // Remove old entries for this file
        beanRegistry.removeByFile(fileUri)
        endpointIndexer.removeByFile(fileUri)

        // Re-scan
        indexFile(file)
    }

    /**
     * Get all discovered beans.
     */
    fun getBeans(): List<SpringBean> = beanRegistry.getAll()

    /**
     * Get all discovered endpoints.
     */
    fun getEndpoints(): List<RestEndpoint> = endpointIndexer.getAll()

    /**
     * Build a bean dependency graph.
     */
    fun getBeanGraph(): BeanGraph {
        return BeanGraphBuilder.build(beanRegistry.getAll())
    }

    private fun indexFile(file: File) {
        val uri = file.toURI().toString()
        val lastModified = file.lastModified()

        // Skip if already indexed and not modified
        val cached = indexedFiles[uri]
        if (cached != null && cached >= lastModified) return

        val content = file.readText()
        val lines = content.lines()

        scanForBeans(uri, lines)
        scanForEndpoints(uri, lines)

        indexedFiles[uri] = lastModified
    }

    private fun scanForBeans(fileUri: String, lines: List<String>) {
        val annotationMap = mapOf(
            "@Component" to BeanType.COMPONENT,
            "@Service" to BeanType.SERVICE,
            "@Repository" to BeanType.REPOSITORY,
            "@Controller" to BeanType.CONTROLLER,
            "@RestController" to BeanType.REST_CONTROLLER,
            "@Configuration" to BeanType.CONFIGURATION,
        )

        var packageName = ""
        var currentAnnotation: BeanType? = null
        var inConfiguration = false
        val dependencies = mutableListOf<String>()

        for ((lineIdx, line) in lines.withIndex()) {
            val trimmed = line.trim()

            // Package declaration
            if (trimmed.startsWith("package ")) {
                packageName = trimmed.removePrefix("package ").trim()
                continue
            }

            // Check for Spring annotations on classes
            for ((annotation, beanType) in annotationMap) {
                if (trimmed.startsWith(annotation)) {
                    currentAnnotation = beanType
                    if (beanType == BeanType.CONFIGURATION) {
                        inConfiguration = true
                    }
                }
            }

            // Class declaration after annotation
            if (currentAnnotation != null && trimmed.startsWith("class ")) {
                val className = extractClassName(trimmed)
                if (className != null) {
                    // Extract constructor dependencies
                    dependencies.clear()
                    extractConstructorDeps(trimmed, dependencies)

                    val qualifiedName = if (packageName.isNotEmpty()) "$packageName.$className" else className
                    beanRegistry.add(SpringBean(
                        name = className.replaceFirstChar { it.lowercase() },
                        qualifiedName = qualifiedName,
                        beanType = currentAnnotation!!,
                        fileUri = fileUri,
                        line = lineIdx,
                        character = line.indexOf("class"),
                        dependencies = dependencies.toList(),
                        scope = "singleton",
                    ))
                }
                currentAnnotation = null
            }

            // @Bean methods inside @Configuration classes
            if (inConfiguration && trimmed.startsWith("@Bean")) {
                // Look for the function on the next few lines
                for (offset in 0..2) {
                    val funcLine = lines.getOrNull(lineIdx + offset)?.trim() ?: continue
                    if (funcLine.startsWith("fun ")) {
                        val methodName = extractMethodName(funcLine)
                        if (methodName != null) {
                            beanRegistry.add(SpringBean(
                                name = methodName,
                                qualifiedName = "$packageName.$methodName",
                                beanType = BeanType.BEAN_METHOD,
                                fileUri = fileUri,
                                line = lineIdx + offset,
                                character = lines[lineIdx + offset].indexOf("fun"),
                                dependencies = emptyList(),
                                scope = "singleton",
                            ))
                        }
                        break
                    }
                }
            }

            // Reset configuration tracking at class end (simplified)
            if (trimmed == "}" && inConfiguration) {
                inConfiguration = false
            }
        }
    }

    private fun scanForEndpoints(fileUri: String, lines: List<String>) {
        val mappingAnnotations = mapOf(
            "@GetMapping" to "GET",
            "@PostMapping" to "POST",
            "@PutMapping" to "PUT",
            "@DeleteMapping" to "DELETE",
            "@PatchMapping" to "PATCH",
            "@RequestMapping" to "REQUEST",
        )

        var classPath = ""
        var currentClass = ""

        for ((lineIdx, line) in lines.withIndex()) {
            val trimmed = line.trim()

            // Class-level @RequestMapping
            if (trimmed.startsWith("@RequestMapping")) {
                classPath = extractMappingPath(trimmed)
            }

            // Track current class
            if (trimmed.contains("class ")) {
                currentClass = extractClassName(trimmed) ?: currentClass
            }

            // Method-level mappings
            for ((annotation, method) in mappingAnnotations) {
                if (trimmed.startsWith(annotation)) {
                    val path = extractMappingPath(trimmed)
                    val fullPath = if (classPath.isNotEmpty()) "$classPath$path" else path

                    // Find the method name
                    for (offset in 0..2) {
                        val funcLine = lines.getOrNull(lineIdx + offset)?.trim() ?: continue
                        if (funcLine.startsWith("fun ") || funcLine.startsWith("suspend fun ")) {
                            val methodName = extractMethodName(funcLine)
                            if (methodName != null) {
                                val httpMethod = if (method == "REQUEST") "GET" else method
                                endpointIndexer.add(RestEndpoint(
                                    method = httpMethod,
                                    path = fullPath.ifEmpty { "/" },
                                    handlerClass = currentClass,
                                    handlerMethod = methodName,
                                    fileUri = fileUri,
                                    line = lineIdx + offset,
                                    character = lines[lineIdx + offset].indexOf("fun"),
                                    parameters = emptyList(),
                                ))
                            }
                            break
                        }
                    }
                }
            }
        }
    }

    private fun extractClassName(line: String): String? {
        val regex = Regex("""class\s+(\w+)""")
        return regex.find(line)?.groupValues?.get(1)
    }

    private fun extractMethodName(line: String): String? {
        val regex = Regex("""fun\s+(\w+)""")
        return regex.find(line)?.groupValues?.get(1)
    }

    private fun extractMappingPath(line: String): String {
        // Match @GetMapping("/path") or @GetMapping(value = "/path")
        val regex = Regex(""""([^"]+)"""")
        return regex.find(line)?.groupValues?.get(1) ?: ""
    }

    private fun extractConstructorDeps(line: String, deps: MutableList<String>) {
        // Simple extraction: find parameter types in constructor
        val regex = Regex("""\b(\w+(?:Service|Repository|Client|Provider|Factory|Mapper))\b""")
        for (match in regex.findAll(line)) {
            deps.add(match.value.replaceFirstChar { it.lowercase() })
        }
    }

    private fun uriToFile(uri: String): File? {
        return try {
            File(URI(uri))
        } catch (e: Exception) {
            // Handle non-URI paths
            File(uri.removePrefix("file://"))
        }
    }
}
