#!/bin/bash
# Commands to initialize git repo and push to remote

# Initialize git repository
git init

# Add all files
git add .

# Create initial commit with conventional commit message
git commit -m "feat: initialize SampleGuard RFID sample integrity tracking system with Rust"

# Add remote repository (replace with your actual repo URL)
# git remote add origin https://github.com/yourusername/sample-guard.git

# Push to remote repository
# git push -u origin main

echo "Repository initialized and committed."
echo "To push to remote, uncomment and update the git remote and git push commands above."

