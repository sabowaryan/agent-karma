# PowerShell script to clean up large files from Git history

# Remove the files from Git tracking but keep them locally
Write-Host "Removing large files from Git tracking..."
git rm --cached -r contracts/contracts/agent-registry/target/

# Create a .gitignore backup just in case
Copy-Item .gitignore .gitignore.backup

# Commit the changes
Write-Host "Committing changes..."
git commit -m "Remove large build artifacts from Git tracking"

# Push the changes
Write-Host "You can now push the changes with: git push origin main"

Write-Host "Cleanup complete! The large files have been removed from Git tracking."
Write-Host "Note: This only affects future commits. To completely remove these files from Git history,"
Write-Host "you would need to use more advanced techniques like git-filter-repo or BFG Repo Cleaner."