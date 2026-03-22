package dev.kode.lsp.gradle

import java.io.File

/**
 * Runs Gradle tasks and streams output.
 */
class TaskRunner(private val projectRoot: File) {

    /**
     * Run a Gradle task and return the output.
     */
    fun runTask(taskName: String): TaskResult {
        return try {
            val connection = org.gradle.tooling.GradleConnector.newConnector()
                .forProjectDirectory(projectRoot)
                .connect()

            val output = StringBuilder()

            connection.use { conn ->
                conn.newBuild()
                    .forTasks(taskName)
                    .setStandardOutput(object : java.io.OutputStream() {
                        override fun write(b: Int) {
                            output.append(b.toChar())
                        }
                        override fun write(b: ByteArray, off: Int, len: Int) {
                            output.append(String(b, off, len))
                        }
                    })
                    .setStandardError(object : java.io.OutputStream() {
                        override fun write(b: Int) {
                            output.append(b.toChar())
                        }
                        override fun write(b: ByteArray, off: Int, len: Int) {
                            output.append(String(b, off, len))
                        }
                    })
                    .run()
            }

            TaskResult(success = true, output = output.toString())
        } catch (e: Exception) {
            TaskResult(success = false, output = "Task '$taskName' failed: ${e.message}")
        }
    }
}

data class TaskResult(
    val success: Boolean,
    val output: String,
)
