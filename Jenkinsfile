node {
   // Clean old workspace
   stage 'Clean workspace'
   deleteDir()

   // Mark the code checkout stage....
   stage 'Checkout'

   // Checkout code from repository
   checkout scm

   // "Run tests"
   stage 'Unit Tests'
   sh "cargo test"
}
