package dev.kode.lsp.spring

import java.util.concurrent.ConcurrentHashMap

/**
 * In-memory store of Spring beans.
 */
class BeanRegistry {

    private val beans = ConcurrentHashMap<String, SpringBean>()

    fun add(bean: SpringBean) {
        beans[bean.qualifiedName] = bean
    }

    fun getAll(): List<SpringBean> = beans.values.toList()

    fun getByName(name: String): SpringBean? {
        return beans.values.find { it.name == name }
    }

    fun removeByFile(fileUri: String) {
        beans.entries.removeIf { it.value.fileUri == fileUri }
    }

    fun clear() {
        beans.clear()
    }

    fun size(): Int = beans.size
}

/**
 * Types of Spring beans.
 */
enum class BeanType {
    COMPONENT,
    SERVICE,
    REPOSITORY,
    CONTROLLER,
    REST_CONTROLLER,
    CONFIGURATION,
    BEAN_METHOD,
}

/**
 * A discovered Spring bean.
 */
data class SpringBean(
    val name: String,
    val qualifiedName: String,
    val beanType: BeanType,
    val fileUri: String,
    val line: Int,
    val character: Int,
    val dependencies: List<String>,
    val scope: String,
)
