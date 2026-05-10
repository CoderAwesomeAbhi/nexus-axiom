#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Make the shell script executable
const binPath = path.join(__dirname, '..', 'bin', 'nexus-axiom.sh');

try {
    if (fs.existsSync(binPath)) {
        fs.chmodSync(binPath, '755');
        console.log('✅ Nexus Axiom wrapper configured');
    }
} catch (err) {
    console.warn('⚠️  Could not set executable permissions:', err.message);
}
