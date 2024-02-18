# 🧹 Path Cleaner

If you've been using your PC for a few years, you've probably encountered a cluttered PATH variable with entries from uninstalled programs or outdated entries from program updates. Path Cleaner is a simple tool designed to help you quickly filter out duplicate and non-existent path entries.

### 🏃‍♂️ Usage
* Open/run `path-cleaner.exe` or put it into PATH and run `path-cleaner` in your terminal of choice.
* Choose the scope (<i>*requires admin privileges</i>)
  * User -> cleans the current users PATH variable
  * System* -> cleans the global PATH variable
  * Both* -> cleans both the global & the current users PATH variable
* That's it🙌

### 🤔 What it does
Path Cleaner removes the following items from the requested scope:
* Non-existant paths
* Duplictes
This process supports variable expansion (e.g. `%WINDIR%`->`C:\Windows`).

### 📝 To-Do
* Prefer path entries with more environment variables over constants

### 🫂 Feel free to contribute
