package dev.kode.lsp.spring

/**
 * Builds a bean dependency graph from the bean registry.
 */
object BeanGraphBuilder {

    fun build(beans: List<SpringBean>): BeanGraph {
        val nodes = beans.map { bean ->
            GraphNode(
                id = bean.name,
                beanType = bean.beanType.name,
                qualifiedName = bean.qualifiedName,
            )
        }

        val edges = mutableListOf<GraphEdge>()
        val beanNames = beans.map { it.name }.toSet()

        for (bean in beans) {
            for (dep in bean.dependencies) {
                if (beanNames.contains(dep)) {
                    edges.add(GraphEdge(from = bean.name, to = dep))
                }
            }
        }

        return BeanGraph(
            nodes = nodes,
            edges = edges,
        )
    }
}

data class BeanGraph(
    val nodes: List<GraphNode>,
    val edges: List<GraphEdge>,
)

data class GraphNode(
    val id: String,
    val beanType: String,
    val qualifiedName: String,
)

data class GraphEdge(
    val from: String,
    val to: String,
)
