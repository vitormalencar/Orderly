# Cleanup previous test environment
Write-Output "Cleaning up previous test environment..."
if (Test-Path "test_folder") {
    Remove-Item -Recurse -Force "test_folder"
    Write-Output "Removed test_folder"
}

# Setup new test environment
Write-Output "Setting up new test environment..."
$paths = @(
    "test_folder/Desktop",
    "test_folder/Desktop/images",
    "test_folder/Downloads",
    "test_folder/Music",
    "test_folder/Videos",
    "test_folder/Documents",
    "test_folder/Pictures/Wallpapers",
    "test_folder/Pictures/Screenshots"
)
$paths | ForEach-Object { New-Item -ItemType Directory -Force -Path $_ }

# Helper function to create file and set timestamp
function New-TestFile {
    param (
        [string]$Path,
        [string]$Timestamp # Format: yyyyMMddHHmm.ss
    )
    New-Item -ItemType File -Path $Path -Force | Out-Null
    $dt = [datetime]::ParseExact($Timestamp, "yyyyMMddHHmm.ss", $null)
    (Get-Item $Path).LastWriteTime = $dt
}

# Create test files on Desktop
$files = @{
    "test_folder/Desktop/file1.txt" = "202401010101.01"
    "test_folder/Desktop/file2.mp3" = "202302020202.02"
    "test_folder/Desktop/file3.mp4" = "202303030303.03"
    "test_folder/Desktop/file4.pdf" = "202304040404.04"
    "test_folder/Desktop/file5.jpg" = "202305050505.05"
    "test_folder/Desktop/file6.png" = "202306060606.06"
    "test_folder/Desktop/file7.gif" = "202307070707.07"
    "test_folder/Desktop/file8.flac" = "202308080808.08"
    "test_folder/Desktop/file9.mov" = "202309090909.09"
    "test_folder/Desktop/file10.docx" = "202310101010.10"
    "test_folder/Desktop/file11.aac" = "202311111111.11"
    "test_folder/Desktop/wallpaper1.jpg" = "202312121212.12"
    "test_folder/Desktop/wallpaper2.png" = "202301010101.01"
    "test_folder/Desktop/screenshot1.png" = "202211111111.11"
    "test_folder/Desktop/screenshot2.jpg" = "202210101010.10"
    "test_folder/Desktop/clearshot1.jpg" = "202212121212.12"
    "test_folder/Desktop/clearshot2.png" = "202213131313.13"
    "test_folder/Desktop/infinite_loop.txt" = "202212121212.12"
    "test_folder/Desktop/images/image1.jpg" = "202301010101.01"
    "test_folder/Desktop/images/image2.png" = "202302020202.02"
    "test_folder/Desktop/images/image3.gif" = "202303030303.03"
    "test_folder/Desktop/images/image4.mp4" = "202304040404.04"
    "test_folder/Desktop/images/image5.pdf" = "202305050505.05"
}

foreach ($file in $files.GetEnumerator()) {
    try {
        New-TestFile -Path $file.Key -Timestamp $file.Value
    } catch {
        Write-Warning "Failed to set timestamp for $($file.Key): $_"
    }
}

Write-Output "Setup complete."
