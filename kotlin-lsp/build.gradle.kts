plugins {
    kotlin("jvm") version "2.1.10"
    application
    id("com.gradleup.shadow") version "9.0.0-beta12"
}

group = "dev.kode"
version = "0.1.0"

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}

repositories {
    mavenCentral()
}

dependencies {
    // LSP4J — standard Java LSP library
    implementation("org.eclipse.lsp4j:org.eclipse.lsp4j:0.23.1")

    // Kotlin compiler for PSI-based source analysis
    implementation("org.jetbrains.kotlin:kotlin-compiler-embeddable:2.1.10")

    // Gradle Tooling API
    implementation("org.gradle:gradle-tooling-api:8.13")

    // YAML parsing for application.yml
    implementation("org.yaml:snakeyaml:2.4")

    // JSON handling
    implementation("com.google.code.gson:gson:2.12.1")

    // Logging
    implementation("org.slf4j:slf4j-api:2.0.17")
    implementation("org.slf4j:slf4j-simple:2.0.17")

    // Test
    testImplementation(kotlin("test"))
}

application {
    mainClass.set("dev.kode.lsp.MainKt")
}

tasks.test {
    useJUnitPlatform()
}

tasks.shadowJar {
    archiveBaseName.set("kode-kotlin-lsp")
    archiveClassifier.set("")
    mergeServiceFiles()
}
