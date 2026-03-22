package dev.kode.lsp.gradle

import java.io.File

/**
 * Connects to a Gradle project using the Gradle Tooling API.
 */
class GradleConnector {

    private var projectRoot: File? = null

    /**
     * Detect a Gradle project at the given root directory.
     */
    fun detectProject(rootDir: File): Boolean {
        val hasGradle = rootDir.resolve("build.gradle.kts").exists()
                || rootDir.resolve("build.gradle").exists()
                || rootDir.resolve("settings.gradle.kts").exists()

        if (hasGradle) {
            projectRoot = rootDir
        }
        return hasGradle
    }

    /**
     * Check if this is a Spring Boot project.
     */
    fun isSpringProject(): Boolean {
        val root = projectRoot ?: return false
        val buildFile = root.resolve("build.gradle.kts").takeIf { it.exists() }
            ?: root.resolve("build.gradle").takeIf { it.exists() }
            ?: return false

        val content = buildFile.readText()
        return content.contains("org.springframework.boot") ||
                content.contains("spring-boot")
    }

    /**
     * List available Gradle tasks.
     */
    fun listTasks(): List<GradleTask> {
        val root = projectRoot ?: return emptyList()
        val tasks = mutableListOf<GradleTask>()

        try {
            val connection = org.gradle.tooling.GradleConnector.newConnector()
                .forProjectDirectory(root)
                .connect()

            connection.use { conn ->
                val model = conn.getModel(org.gradle.tooling.model.GradleProject::class.java)
                for (task in model.tasks) {
                    tasks.add(GradleTask(
                        name = task.name,
                        path = task.path,
                        description = task.description,
                        group = task.group,
                    ))
                }
            }
        } catch (e: Exception) {
            // Fallback: return common tasks
            tasks.addAll(defaultTasks())
        }

        return tasks
    }

    private fun defaultTasks(): List<GradleTask> {
        return listOf(
            GradleTask("build", ":build", "Assembles and tests this project", "build"),
            GradleTask("clean", ":clean", "Deletes the build directory", "build"),
            GradleTask("test", ":test", "Runs the tests", "verification"),
            GradleTask("bootRun", ":bootRun", "Runs Spring Boot application", "application"),
            GradleTask("bootJar", ":bootJar", "Assembles an executable jar", "build"),
        )
    }
}

/**
 * A Gradle task.
 */
data class GradleTask(
    val name: String,
    val path: String,
    val description: String?,
    val group: String?,
)
