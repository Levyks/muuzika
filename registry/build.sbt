ThisBuild / version := "0.1.0-SNAPSHOT"

ThisBuild / scalaVersion := "3.3.5"

lazy val root = (project in file("."))
  .settings(
    name := "registry"
  )

lazy val pekkoVersion = "1.1.3"
lazy val pekkoGrpcVersion = "1.1.1"

enablePlugins(PekkoGrpcPlugin)

// Run in a separate JVM, to make sure sbt waits until all threads have
// finished before returning.
// If you want to keep the application running while executing other
// sbt tasks, consider https://github.com/spray/sbt-revolver/
fork := true

libraryDependencies ++= Seq(
  "org.apache.pekko" %% "pekko-actor-typed" % pekkoVersion,
  "org.apache.pekko" %% "pekko-stream-typed" % pekkoVersion,
  "org.apache.pekko" %% "pekko-discovery" % pekkoVersion,
  "org.apache.pekko" %% "pekko-pki" % pekkoVersion,

  "com.thesamet.scalapb" %% "scalapb-validate-core" % scalapb.validate.compiler.BuildInfo.version % "protobuf",

  "ch.qos.logback" % "logback-classic" % "1.5.16",

  "org.apache.pekko" %% "pekko-actor-testkit-typed" % pekkoVersion % Test,
  "org.apache.pekko" %% "pekko-stream-testkit" % pekkoVersion % Test,
  "org.scalatest" %% "scalatest" % "3.2.19" % Test,
  "org.scalatestplus" %% "mockito-5-12" % "3.2.19.0" % Test
)

Compile / PB.protoSources += file("../proto")
Compile / PB.targets +=
  scalapb.validate.gen(scalapb.GeneratorOption.FlatPackage,
  ) -> (Compile / pekkoGrpcCodeGeneratorSettings / target).value

pekkoGrpcGeneratedSources := Seq(PekkoGrpc.Server)