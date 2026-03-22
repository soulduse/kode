package dev.kode.lsp.spring

/**
 * Provides auto-completion for Spring Boot application.yml/application.properties.
 *
 * Reads spring-configuration-metadata.json from the classpath to provide
 * property key suggestions with types and default values.
 */
class YamlCompleter {

    private val properties = mutableListOf<ConfigProperty>()

    init {
        // Load built-in Spring Boot common properties
        loadBuiltinProperties()
    }

    /**
     * Get completion suggestions for a given YAML path prefix.
     */
    fun complete(prefix: String): List<ConfigProperty> {
        if (prefix.isEmpty()) {
            return properties.take(50) // Return top-level suggestions
        }
        return properties.filter { it.name.startsWith(prefix) }
    }

    /**
     * Load properties from spring-configuration-metadata.json on the classpath.
     */
    fun loadFromMetadata(json: String) {
        try {
            val parsed = ConfigMetadataParser.parse(json)
            properties.addAll(parsed)
        } catch (e: Exception) {
            // Silently ignore parse errors
        }
    }

    fun getAll(): List<ConfigProperty> = properties.toList()

    /**
     * Built-in common Spring Boot properties for offline completion.
     */
    private fun loadBuiltinProperties() {
        val builtins = listOf(
            ConfigProperty("server.port", "java.lang.Integer", "8080", "Server HTTP port"),
            ConfigProperty("server.servlet.context-path", "java.lang.String", null, "Context path of the application"),
            ConfigProperty("server.address", "java.net.InetAddress", null, "Network address to which the server should bind"),
            ConfigProperty("spring.application.name", "java.lang.String", null, "Application name"),
            ConfigProperty("spring.profiles.active", "java.lang.String", null, "Active Spring profiles"),
            ConfigProperty("spring.datasource.url", "java.lang.String", null, "JDBC URL of the database"),
            ConfigProperty("spring.datasource.username", "java.lang.String", null, "Login username of the database"),
            ConfigProperty("spring.datasource.password", "java.lang.String", null, "Login password of the database"),
            ConfigProperty("spring.datasource.driver-class-name", "java.lang.String", null, "JDBC driver class name"),
            ConfigProperty("spring.jpa.hibernate.ddl-auto", "java.lang.String", null, "DDL mode (none, validate, update, create, create-drop)"),
            ConfigProperty("spring.jpa.show-sql", "java.lang.Boolean", "false", "Enable logging of SQL statements"),
            ConfigProperty("spring.jpa.database-platform", "java.lang.String", null, "Hibernate dialect"),
            ConfigProperty("spring.jackson.serialization.indent-output", "java.lang.Boolean", "false", "Indent JSON output"),
            ConfigProperty("spring.cache.type", "java.lang.String", null, "Cache type (simple, redis, caffeine, etc.)"),
            ConfigProperty("spring.redis.host", "java.lang.String", "localhost", "Redis server host"),
            ConfigProperty("spring.redis.port", "java.lang.Integer", "6379", "Redis server port"),
            ConfigProperty("logging.level.root", "java.lang.String", "INFO", "Root logging level"),
            ConfigProperty("logging.file.name", "java.lang.String", null, "Log file name"),
            ConfigProperty("management.endpoints.web.exposure.include", "java.lang.String", null, "Actuator endpoints to expose"),
            ConfigProperty("management.endpoint.health.show-details", "java.lang.String", "never", "Show health details"),
        )
        properties.addAll(builtins)
    }
}

/**
 * A Spring Boot configuration property.
 */
data class ConfigProperty(
    val name: String,
    val type: String,
    val defaultValue: String?,
    val description: String?,
)
