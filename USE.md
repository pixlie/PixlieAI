# Using Pixlie AI
Pixlie AI is built to be run by anyone on their own computer or on the cloud. You will need a recent graphics card to run everything localy.

Alternatively, you can use a mix of cloud services for the GPU intensive tasks.

At this time you cannot download Pixlie. This feature will be available in the near future, but if you are eager to test it out before you can download it, this guide will walk you through how to set up Pixlie. You don't need to be a developer to follow along, but some basic comfort with using a terminal/command prompt will be helpful.

## System Requirements

- **Mac**: macOS 10.15 (Catalina) or newer
- **Windows**: Windows 10 or newer
- At least 4GB of RAM
- At least 2GB of free disk space

## Step 1: Install Required Software

### For Mac Users

1. **Install Homebrew** (a package manager for Mac)
   
   Open Terminal (you can find it using Spotlight by pressing Cmd+Space and typing "Terminal"), then paste the following command and press Enter:

   ```
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

   Follow the on-screen instructions to complete the installation.

2. **Install Required Dependencies**

   In Terminal, enter:

   ```
   brew install git rust node python
   ```

3. **Install pnpm** (a faster alternative to npm)

   ```
   npm install -g pnpm
   ```

### For Windows Users

1. **Install Git for Windows**
   
   Download and install from https://git-scm.com/download/win

2. **Install Rust for Windows**
   
   Download and run the installer from https://www.rust-lang.org/tools/install

3. **Install Node.js for Windows**
   
   Download and install the LTS version from https://nodejs.org/

4. **Install pnpm**

   Open Command Prompt or PowerShell and run:
   ```
   npm install -g pnpm
   ```

## Step 2: Get the Pixlie AI Code

1. **Clone the Repository**

   In your terminal (Mac) or Command Prompt/PowerShell (Windows), run:

   ```
   git clone https://github.com/pixlie/PixlieAI.git
   cd pixlie-ai
   ```
## Step 3: Run Pixlie AI

You'll need to run both the backend and frontend components in separate terminal windows.

### Run the Backend

1. **Open a Terminal Window**

   On Mac, open Terminal. On Windows, open Command Prompt or PowerShell.

2. **Navigate to the pixlie_ai directory**

   ```
   cd pixlie-ai/pixlie_ai
   ```

3. **Start the Backend**

   On Mac:
   ```
   RUST_LOG=info cargo run --bin cli
   ```

   On Windows (Command Prompt):
   ```
   set RUST_LOG=info && cargo run --bin cli
   ```

   On Windows (PowerShell):
   ```
   $env:RUST_LOG="info"; cargo run --bin cli
   ```

   This will start the Pixlie AI backend. The first run might take several minutes as it compiles the Rust code.

### Run the Frontend (Web UI)

1. **Open Another Terminal Window**

   Open a new Terminal (Mac) or Command Prompt/PowerShell (Windows) window.

2. **Navigate to the admin directory**

   ```
   cd pixlie-ai/admin
   ```

3. **Install Dependencies and Start the Frontend**

   ```
   pnpm install
   pnpm run dev
   ```

   The first run might take a minute or two as it installs all required dependencies.

## Step 4: Access the Pixlie AI Web Interface

Once both the backend and frontend are running:

1. Open your web browser
2. Navigate to http://localhost:5173
3. You should now see the Pixlie AI interface!

## Troubleshooting

### Common Issues on Mac

- **"Command not found" errors**: Make sure you've installed Homebrew correctly and that it's in your PATH.

### Common Issues on Windows

- **Path not found errors**: Make sure you're navigating to the correct directories.
- **Environment variable issues**: In Command Prompt, use `set RUST_LOG=info` instead of the Linux-style `RUST_LOG=info`.
- **Rust compilation errors**: Make sure you've installed the Visual C++ Build Tools during Rust installation.

### Issues That May Require Additional Libraries

If you encounter compilation errors mentioning missing libraries:

- **On Mac**: Run `brew install rocksdb clang`
- **On Windows**: You may need to install Visual C++ Build Tools from the Visual Studio Installer

## Stopping Pixlie AI

When you're done using Pixlie AI:

1. Press Ctrl+C in both terminal windows to stop the backend and frontend servers.

## Getting Updates

To update your local copy of Pixlie AI when new features are released:

```
cd pixlie-ai
git pull
```

Then restart both the backend and frontend as described in Step 3.

## Need Help?

If you encounter any issues not covered in this guide, feel free to reach out at team@pixlie.com for support.
