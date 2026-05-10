#!/usr/bin/env node

const os = require('os');
const { execSync } = require('child_process');

console.log('🛡️  Nexus Axiom - eBPF Security Installation');
console.log('='.repeat(50));

// Check OS
if (os.platform() !== 'linux') {
    console.error('❌ Nexus Axiom only supports Linux (kernel 5.8+)');
    console.error('   Current OS:', os.platform());
    process.exit(1);
}

// Check architecture
const arch = os.arch();
if (arch !== 'x64' && arch !== 'arm64') {
    console.error('❌ Unsupported architecture:', arch);
    console.error('   Supported: x64, arm64');
    process.exit(1);
}

console.log('✅ OS Check: Linux');
console.log('✅ Architecture:', arch);
console.log('');
console.log('📦 Nexus Axiom has been added to your project.');
console.log('');
console.log('To install the system binary, run:');
console.log('  npx nexus-axiom install');
console.log('');
console.log('Or install directly:');
console.log('  curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash');
console.log('');
console.log('Documentation: https://github.com/CoderAwesomeAbhi/nexus-axiom');
