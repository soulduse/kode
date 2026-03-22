package dev.kode.lsp.spring

import com.google.gson.Gson
import com.google.gson.JsonObject

/**
 * Parses spring-configuration-metadata.json files found in Spring Boot JARs.
 *
 * These files are located at META-INF/spring-configuration-metadata.json
 * inside Spring Boot starter JARs and provide property definitions.
 */
object ConfigMetadataParser {

    private val gson = Gson()

    /**
     * Parse a spring-configuration-metadata.json string into ConfigProperty list.
     */
    fun parse(json: String): List<ConfigProperty> {
        val root = gson.fromJson(json, JsonObject::class.java) ?: return emptyList()
        val properties = mutableListOf<ConfigProperty>()

        val propsArray = root.getAsJsonArray("properties") ?: return emptyList()

        for (element in propsArray) {
            val obj = element.asJsonObject
            val name = obj.get("name")?.asString ?: continue
            val type = obj.get("type")?.asString ?: "java.lang.String"
            val defaultValue = obj.get("defaultValue")?.asString
            val description = obj.get("description")?.asString

            properties.add(ConfigProperty(
                name = name,
                type = type,
                defaultValue = defaultValue,
                description = description,
            ))
        }

        return properties
    }
}
