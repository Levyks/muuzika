addSbtPlugin("org.apache.pekko" % "pekko-grpc-sbt-plugin" % "1.1.1")

addSbtPlugin("com.github.sbt" % "sbt-javaagent" % "0.1.8")

libraryDependencies ++= Seq(
  "com.thesamet.scalapb" %% "compilerplugin"           % "0.11.17",
  "com.thesamet.scalapb" %% "scalapb-validate-codegen" % "0.3.6"
)

