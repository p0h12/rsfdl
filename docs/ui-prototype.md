# UI Prototyp

Interaktiver HTML/CSS-Prototyp der rsfdl-Oberfläche. Die Tailwind-Klassen sind 1:1 identisch mit den Dioxus-Komponenten in `crates/gui/src/`.

<!-- Tab Navigation -->
<div class="proto" style="margin-top: 1rem;">
<div class="flex gap-2 flex-wrap mb-4">
  <button class="proto-tab px-3 py-1.5 rounded text-sm font-medium cursor-pointer bg-blue-600 text-white" data-tab="state-empty" onclick="showState('state-empty')">Leer</button>
  <button class="proto-tab px-3 py-1.5 rounded text-sm font-medium cursor-pointer bg-gray-200 text-gray-700" data-tab="state-password" onclick="showState('state-password')">Passwort</button>
  <button class="proto-tab px-3 py-1.5 rounded text-sm font-medium cursor-pointer bg-gray-200 text-gray-700" data-tab="state-loaded" onclick="showState('state-loaded')">Geladen</button>
  <button class="proto-tab px-3 py-1.5 rounded text-sm font-medium cursor-pointer bg-gray-200 text-gray-700" data-tab="state-downloading" onclick="showState('state-downloading')">Download</button>
  <button class="proto-tab px-3 py-1.5 rounded text-sm font-medium cursor-pointer bg-gray-200 text-gray-700" data-tab="state-done" onclick="showState('state-done')">Fertig</button>
  <button class="proto-tab px-3 py-1.5 rounded text-sm font-medium cursor-pointer bg-gray-200 text-gray-700" data-tab="state-settings" onclick="showState('state-settings')">Settings</button>
  <button class="proto-tab px-3 py-1.5 rounded text-sm font-medium cursor-pointer bg-gray-200 text-gray-700" data-tab="state-creator" onclick="showState('state-creator')">Creator</button>
</div>
</div>

<!-- ============================================================ -->
<!-- State 1: Empty -->
<!-- ============================================================ -->
<div id="state-empty" class="proto proto-state flex-col overflow-hidden" style="height: 430px;">
  <div class="window-chrome"><span class="window-dot window-dot-close"></span><span class="window-dot window-dot-min"></span><span class="window-dot window-dot-max"></span></div>
  <div class="flex items-center justify-between px-4 py-3 bg-gray-800 text-white">
    <h1 class="text-lg font-bold">rsfdl</h1>
    <div class="flex gap-2">
      <button class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium">Open File</button>
      <button class="px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-sm font-medium">Create</button>
      <button class="px-3 py-1.5 bg-gray-600 hover:bg-gray-700 rounded text-sm">Settings</button>
    </div>
  </div>

  <!-- Empty state (main_view.rs) -->
  <div class="flex-1 flex items-center justify-center text-gray-400">
    <p class="text-lg">Open an .sfdl file to begin</p>
  </div>

</div>

<!-- ============================================================ -->
<!-- State 2: Password Dialog -->
<!-- ============================================================ -->
<div id="state-password" class="proto proto-state flex-col overflow-hidden" style="height: 430px; display: none; position: relative;">
  <div class="window-chrome"><span class="window-dot window-dot-close"></span><span class="window-dot window-dot-min"></span><span class="window-dot window-dot-max"></span></div>
  <div class="flex items-center justify-between px-4 py-3 bg-gray-800 text-white">
    <h1 class="text-lg font-bold">rsfdl</h1>
    <div class="flex gap-2">
      <button class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium opacity-50 cursor-not-allowed">Open File</button>
      <button class="px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-sm font-medium opacity-50 cursor-not-allowed">Create</button>
      <button class="px-3 py-1.5 bg-gray-600 hover:bg-gray-700 rounded text-sm">Settings</button>
    </div>
  </div>

  <!-- Backdrop + Dialog (password_dialog.rs) -->
  <div class="flex-1 relative">
    <div class="absolute inset-0 bg-black/50 flex items-center justify-center">
      <div class="bg-white rounded-lg shadow-xl p-6 w-96 space-y-4">
        <h2 class="text-lg font-bold text-gray-900">Password Required</h2>
        <p class="text-sm text-gray-600">This container is encrypted.</p>
        <input class="w-full px-3 py-2 border rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-500" type="password" placeholder="Enter password...">
        <div class="flex justify-end gap-2">
          <button class="px-3 py-1.5 bg-gray-200 hover:bg-gray-300 rounded text-sm">Cancel</button>
          <button class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm font-medium">Decrypt</button>
        </div>
      </div>
    </div>
  </div>

</div>

<!-- ============================================================ -->
<!-- State 3: Container Loaded -->
<!-- ============================================================ -->
<div id="state-loaded" class="proto proto-state flex-col overflow-hidden" style="height: 530px; display: none;">
  <div class="window-chrome"><span class="window-dot window-dot-close"></span><span class="window-dot window-dot-min"></span><span class="window-dot window-dot-max"></span></div>
  <div class="flex items-center justify-between px-4 py-3 bg-gray-800 text-white">
    <h1 class="text-lg font-bold">rsfdl</h1>
    <div class="flex gap-2">
      <button class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium">Open File</button>
      <button class="px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-sm font-medium">Create</button>
      <button class="px-3 py-1.5 bg-gray-600 hover:bg-gray-700 rounded text-sm">Settings</button>
    </div>
  </div>

  <!-- Container Info (container_info.rs) -->
  <div class="px-4 py-3 bg-gray-50 border-b text-sm space-y-1">
    <p class="font-medium text-gray-900">Example Release v2.1</p>
    <div class="flex flex-wrap gap-x-6 gap-y-1 text-gray-600">
      <span>Server: ftp.example.com:21</span>
      <span>Uploader: user123</span>
      <span>Files: 5/8</span>
      <span>Selected: 1.2 GB / 2.4 GB</span>
    </div>
  </div>

  <!-- File List (file_list.rs + file_row.rs) -->
  <div class="flex-1 overflow-y-auto">
    <!-- Select All -->
    <div class="flex items-center px-4 py-2 bg-gray-100 border-b text-sm font-medium text-gray-700">
      <input type="checkbox" class="mr-3" checked>
      <span>Select All</span>
    </div>
    <!-- Package header -->
    <div class="flex items-center px-4 py-1.5 bg-gray-50 border-b text-sm font-medium text-gray-600">
      <input type="checkbox" class="mr-3" checked>
      <span>Package.1</span>
    </div>
    <!-- File rows -->
    <div class="flex items-center px-4 py-1.5 hover:bg-gray-50 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3" checked>
      <span class="flex-1 truncate text-gray-800">movie.part01.rar</span>
      <span class="w-24 text-right text-gray-500 mr-4">512 MB</span>
      <span class="w-24 text-right text-gray-400"></span>
    </div>
    <div class="flex items-center px-4 py-1.5 hover:bg-gray-50 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3" checked>
      <span class="flex-1 truncate text-gray-800">movie.part02.rar</span>
      <span class="w-24 text-right text-gray-500 mr-4">512 MB</span>
      <span class="w-24 text-right text-gray-400"></span>
    </div>
    <div class="flex items-center px-4 py-1.5 hover:bg-gray-50 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3" checked>
      <span class="flex-1 truncate text-gray-800">movie.part03.rar</span>
      <span class="w-24 text-right text-gray-500 mr-4">256 MB</span>
      <span class="w-24 text-right text-gray-400"></span>
    </div>
    <div class="flex items-center px-4 py-1.5 hover:bg-gray-50 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3">
      <span class="flex-1 truncate text-gray-800">movie.nfo</span>
      <span class="w-24 text-right text-gray-500 mr-4">2 KB</span>
      <span class="w-24 text-right text-gray-400"></span>
    </div>
    <div class="flex items-center px-4 py-1.5 hover:bg-gray-50 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3">
      <span class="flex-1 truncate text-gray-800">cover.jpg</span>
      <span class="w-24 text-right text-gray-500 mr-4">450 KB</span>
      <span class="w-24 text-right text-gray-400"></span>
    </div>
  </div>

  <!-- Action buttons (main_view.rs) -->
  <div class="px-4 py-3 border-t bg-white flex justify-center gap-3">
    <button class="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded text-sm font-medium">Start Download</button>
  </div>

</div>

<!-- ============================================================ -->
<!-- State 4: Downloading -->
<!-- ============================================================ -->
<div id="state-downloading" class="proto proto-state flex-col overflow-hidden" style="height: 580px; display: none;">
  <div class="window-chrome"><span class="window-dot window-dot-close"></span><span class="window-dot window-dot-min"></span><span class="window-dot window-dot-max"></span></div>
  <div class="flex items-center justify-between px-4 py-3 bg-gray-800 text-white">
    <h1 class="text-lg font-bold">rsfdl</h1>
    <div class="flex gap-2">
      <button class="px-3 py-1.5 bg-blue-600 rounded text-sm font-medium opacity-50 cursor-not-allowed">Open File</button>
      <button class="px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-sm font-medium opacity-50 cursor-not-allowed">Create</button>
      <button class="px-3 py-1.5 bg-gray-600 hover:bg-gray-700 rounded text-sm">Settings</button>
    </div>
  </div>

  <!-- Container Info -->
  <div class="px-4 py-3 bg-gray-50 border-b text-sm space-y-1">
    <p class="font-medium text-gray-900">Example Release v2.1</p>
    <div class="flex flex-wrap gap-x-6 gap-y-1 text-gray-600">
      <span>Server: ftp.example.com:21</span>
      <span>Files: 3/5</span>
      <span>Selected: 1.2 GB / 1.2 GB</span>
    </div>
  </div>

  <!-- File List (truncated, scrollable) -->
  <div class="flex-1 overflow-y-auto">
    <div class="flex items-center px-4 py-2 bg-gray-100 border-b text-sm font-medium text-gray-700">
      <input type="checkbox" class="mr-3" checked disabled>
      <span>Select All</span>
    </div>
    <div class="flex items-center px-4 py-1.5 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3" checked disabled>
      <span class="flex-1 truncate text-gray-800">movie.part01.rar</span>
      <span class="w-24 text-right text-gray-500 mr-4">512 MB</span>
      <span class="w-24 text-right text-green-600">completed</span>
    </div>
    <div class="flex items-center px-4 py-1.5 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3" checked disabled>
      <span class="flex-1 truncate text-gray-800">movie.part02.rar</span>
      <span class="w-24 text-right text-gray-500 mr-4">512 MB</span>
      <span class="w-24 text-right text-blue-600">downloading</span>
    </div>
    <div class="flex items-center px-4 py-1.5 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3" checked disabled>
      <span class="flex-1 truncate text-gray-800">movie.part03.rar</span>
      <span class="w-24 text-right text-gray-500 mr-4">256 MB</span>
      <span class="w-24 text-right text-gray-400"></span>
    </div>
  </div>

  <!-- Cancel button -->
  <div class="px-4 py-3 border-t bg-white flex justify-center gap-3">
    <button class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded text-sm font-medium">Cancel</button>
  </div>

  <!-- Progress Panel (progress_panel.rs) -->
  <div class="border-t bg-white">
    <!-- Global progress -->
    <div class="px-4 py-2 bg-gray-100 border-b">
      <div class="flex justify-between text-sm text-gray-700 mb-1">
        <span class="font-medium">1/3 files</span>
        <span class="text-gray-500">680 MB / 1.2 GB  12.5 MB/s  ETA 0:42</span>
      </div>
      <div class="w-full bg-gray-200 rounded-full h-2">
        <div class="h-2 rounded-full bg-cyan-500" style="width: 56%"></div>
      </div>
    </div>
    <!-- Per-file progress -->
    <div class="max-h-48 overflow-y-auto">
      <div class="px-4 py-1.5 border-b border-gray-100 text-sm">
        <div class="flex justify-between items-center mb-1">
          <span class="truncate text-gray-800 mr-2">movie.part02.rar</span>
          <div class="flex items-center gap-2 whitespace-nowrap">
            <span class="text-gray-500">168 MB / 512 MB</span>
            <button class="text-red-500 hover:text-red-700 text-xs font-medium px-1">X</button>
          </div>
        </div>
        <div class="w-full bg-gray-200 rounded-full h-1.5">
          <div class="h-1.5 rounded-full bg-blue-500" style="width: 33%"></div>
        </div>
      </div>
      <div class="px-4 py-1.5 border-b border-gray-100 text-sm">
        <div class="flex justify-between items-center mb-1">
          <span class="truncate text-gray-800 mr-2">movie.part01.rar</span>
          <span class="text-gray-500">completed</span>
        </div>
      </div>
    </div>
  </div>

</div>

<!-- ============================================================ -->
<!-- State 5: Done -->
<!-- ============================================================ -->
<div id="state-done" class="proto proto-state flex-col overflow-hidden" style="height: 530px; display: none;">
  <div class="window-chrome"><span class="window-dot window-dot-close"></span><span class="window-dot window-dot-min"></span><span class="window-dot window-dot-max"></span></div>
  <div class="flex items-center justify-between px-4 py-3 bg-gray-800 text-white">
    <h1 class="text-lg font-bold">rsfdl</h1>
    <div class="flex gap-2">
      <button class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium">Open File</button>
      <button class="px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-sm font-medium">Create</button>
      <button class="px-3 py-1.5 bg-gray-600 hover:bg-gray-700 rounded text-sm">Settings</button>
    </div>
  </div>

  <!-- Container Info -->
  <div class="px-4 py-3 bg-gray-50 border-b text-sm space-y-1">
    <p class="font-medium text-gray-900">Example Release v2.1</p>
    <div class="flex flex-wrap gap-x-6 gap-y-1 text-gray-600">
      <span>Server: ftp.example.com:21</span>
      <span>Files: 3/3</span>
      <span>Selected: 1.2 GB / 1.2 GB</span>
    </div>
  </div>

  <!-- File List -->
  <div class="flex-1 overflow-y-auto">
    <div class="flex items-center px-4 py-2 bg-gray-100 border-b text-sm font-medium text-gray-700">
      <input type="checkbox" class="mr-3" checked disabled>
      <span>Select All</span>
    </div>
    <div class="flex items-center px-4 py-1.5 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3" checked disabled>
      <span class="flex-1 truncate text-gray-800">movie.part01.rar</span>
      <span class="w-24 text-right text-gray-500 mr-4">512 MB</span>
      <span class="w-24 text-right text-green-600">completed</span>
    </div>
    <div class="flex items-center px-4 py-1.5 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3" checked disabled>
      <span class="flex-1 truncate text-gray-800">movie.part02.rar</span>
      <span class="w-24 text-right text-gray-500 mr-4">512 MB</span>
      <span class="w-24 text-right text-green-600">completed</span>
    </div>
    <div class="flex items-center px-4 py-1.5 text-sm border-b border-gray-100">
      <input type="checkbox" class="mr-3" checked disabled>
      <span class="flex-1 truncate text-gray-800">movie.part03.rar</span>
      <span class="w-24 text-right text-gray-500 mr-4">256 MB</span>
      <span class="w-24 text-right text-red-600">failed</span>
    </div>
  </div>

  <!-- Reset button -->
  <div class="px-4 py-3 border-t bg-white flex justify-center gap-3">
    <button class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm font-medium">Reset</button>
  </div>

  <!-- Summary Banner (summary_banner.rs) -->
  <div class="px-4 py-3 text-sm font-medium bg-red-100 text-red-800">
    Done: 3 total, 2 completed, 0 skipped, 1 failed, 0 cancelled
  </div>

  <!-- Progress Panel (final state) -->
  <div class="border-t bg-white">
    <div class="px-4 py-2 bg-gray-100 border-b">
      <div class="flex justify-between text-sm text-gray-700 mb-1">
        <span class="font-medium">3/3 files</span>
        <span class="text-gray-500">1.1 GB / 1.2 GB</span>
      </div>
      <div class="w-full bg-gray-200 rounded-full h-2">
        <div class="h-2 rounded-full bg-cyan-500" style="width: 92%"></div>
      </div>
    </div>
    <div class="max-h-48 overflow-y-auto">
      <div class="px-4 py-1.5 border-b border-gray-100 text-sm">
        <div class="flex justify-between items-center">
          <span class="truncate text-gray-800 mr-2">movie.part01.rar</span>
          <span class="text-gray-500">completed</span>
        </div>
      </div>
      <div class="px-4 py-1.5 border-b border-gray-100 text-sm">
        <div class="flex justify-between items-center">
          <span class="truncate text-gray-800 mr-2">movie.part02.rar</span>
          <span class="text-gray-500">completed</span>
        </div>
      </div>
      <div class="px-4 py-1.5 border-b border-gray-100 text-sm">
        <div class="flex justify-between items-center">
          <span class="truncate text-gray-800 mr-2">movie.part03.rar</span>
          <span class="text-red-500">Connection timed out</span>
        </div>
      </div>
    </div>
  </div>

</div>

<div id="state-settings" class="proto proto-state flex-col overflow-hidden" style="height: 630px; display: none;">
  <div class="window-chrome"><span class="window-dot window-dot-close"></span><span class="window-dot window-dot-min"></span><span class="window-dot window-dot-max"></span></div>
  <div class="flex items-center justify-between px-4 py-3 bg-gray-800 text-white">
    <h1 class="text-lg font-bold">rsfdl</h1>
    <div class="flex gap-2">
      <button class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium">Open File</button>
      <button class="px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-sm font-medium">Create</button>
      <button class="px-3 py-1.5 bg-gray-600 hover:bg-gray-700 rounded text-sm">Settings</button>
    </div>
  </div>
  <div class="flex items-center justify-between px-4 py-3 bg-gray-100 border-b">
    <h2 class="text-lg font-bold text-gray-900">Settings</h2>
    <div class="flex items-center gap-2">
      <button class="px-4 py-1.5 bg-green-600 hover:bg-green-700 text-white rounded text-sm font-medium">Save</button>
      <button class="px-3 py-1.5 bg-gray-200 hover:bg-gray-300 rounded text-sm">Back</button>
    </div>
  </div>
  <div class="flex-1 overflow-y-auto p-6 space-y-6">
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Download Directory</label>
      <div class="flex gap-2">
        <input class="flex-1 px-3 py-2 border rounded text-sm bg-gray-50" type="text" readonly value="/Users/user/Downloads">
        <button class="px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm">Browse...</button>
      </div>
    </div>
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Max Download Threads</label>
        <input class="w-24 px-3 py-2 border rounded text-sm" type="number" value="3" min="1" max="10">
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Max Retries</label>
        <input class="w-24 px-3 py-2 border rounded text-sm" type="number" value="3" min="0" max="10">
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Retry Wait (seconds)</label>
        <input class="w-24 px-3 py-2 border rounded text-sm" type="number" value="5" min="1" max="120">
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">FTP Timeout (seconds)</label>
        <input class="w-24 px-3 py-2 border rounded text-sm" type="number" value="30" min="5" max="300">
      </div>
    </div>
    <div class="space-y-3">
      <label class="flex items-center gap-3 cursor-pointer">
        <input type="checkbox" class="w-4 h-4" checked>
        <div>
          <span class="block text-sm font-medium text-gray-700">Resume Downloads</span>
          <span class="block text-xs text-gray-500">Skip files that are already fully downloaded</span>
        </div>
      </label>
      <label class="flex items-center gap-3 cursor-pointer">
        <input type="checkbox" class="w-4 h-4" checked>
        <div>
          <span class="block text-sm font-medium text-gray-700">Create Package Subfolder</span>
          <span class="block text-xs text-gray-500">Create a subfolder per package in the download directory</span>
        </div>
      </label>
    </div>
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Auto Password List</label>
      <p class="text-xs text-gray-500 mb-2">One password per line. Tried automatically when opening encrypted containers.</p>
      <div class="w-full px-3 py-2 border rounded text-sm font-mono h-24 bg-white overflow-auto whitespace-pre text-gray-800">password1
secret123</div>
    </div>
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">File Exclusion Patterns</label>
      <p class="text-xs text-gray-500 mb-2">One glob pattern per line. Matching files are excluded from download.</p>
      <div class="w-full px-3 py-2 border rounded text-sm font-mono h-24 bg-white overflow-auto whitespace-pre text-gray-800">*.nfo
*.jpg</div>
    </div>
  </div>
</div>

<!-- ============================================================ -->
<!-- State 7: Creator -->
<!-- ============================================================ -->
<div id="state-creator" class="proto proto-state flex-col overflow-hidden" style="height: 630px; display: none;">
  <div class="window-chrome"><span class="window-dot window-dot-close"></span><span class="window-dot window-dot-min"></span><span class="window-dot window-dot-max"></span></div>
  <div class="flex items-center justify-between px-4 py-3 bg-gray-800 text-white">
    <h1 class="text-lg font-bold">rsfdl</h1>
    <div class="flex gap-2">
      <button class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium">Open File</button>
      <button class="px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-sm font-medium">Create</button>
      <button class="px-3 py-1.5 bg-gray-600 hover:bg-gray-700 rounded text-sm">Settings</button>
    </div>
  </div>

  <!-- Sub-header (gleicher Stil wie Settings) -->
  <div class="flex items-center justify-between px-4 py-3 bg-gray-100 border-b">
    <h2 class="text-lg font-bold text-gray-900">Create SFDL</h2>
    <div class="flex items-center gap-2">
      <button class="px-4 py-1.5 bg-green-600 hover:bg-green-700 text-white rounded text-sm font-medium">Create SFDL</button>
      <button class="px-3 py-1.5 bg-gray-200 hover:bg-gray-300 rounded text-sm">Back</button>
    </div>
  </div>
  <!-- Form (scrollbar) -->
  <div class="flex-1 overflow-y-auto p-6 space-y-6">
    <!-- FTP Connection -->
    <div>
      <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-3">FTP Connection</h3>
      <div class="grid grid-cols-3 gap-4">
        <div class="col-span-2">
          <label class="block text-sm font-medium text-gray-700 mb-1">Host</label>
          <input class="w-full px-3 py-2 border rounded text-sm" type="text" placeholder="ftp.example.com">
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Port</label>
          <input class="w-24 px-3 py-2 border rounded text-sm" type="number" value="21">
        </div>
      </div>
      <div class="grid grid-cols-2 gap-4 mt-3">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Username</label>
          <input class="w-full px-3 py-2 border rounded text-sm" type="text" placeholder="ftpuser">
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Password</label>
          <input class="w-full px-3 py-2 border rounded text-sm" type="password" placeholder="••••••">
        </div>
      </div>
    </div>
    <!-- Content -->
    <div>
      <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-3">Content</h3>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Remote Path</label>
        <input class="w-full px-3 py-2 border rounded text-sm" type="text" placeholder="/releases/movie/">
      </div>
      <div class="flex gap-4 mt-3">
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="radio" name="mode" checked class="w-4 h-4">
          <div>
            <span class="text-sm font-medium text-gray-700">BulkFolder</span>
            <span class="block text-xs text-gray-500">Store path only, no FTP listing</span>
          </div>
        </label>
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="radio" name="mode" class="w-4 h-4">
          <div>
            <span class="text-sm font-medium text-gray-700">FileList</span>
            <span class="block text-xs text-gray-500">Connect to FTP and list files with sizes</span>
          </div>
        </label>
      </div>
    </div>
    <!-- Metadata -->
    <div>
      <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-3">Metadata</h3>
      <div class="grid grid-cols-3 gap-4">
        <div class="col-span-2">
          <label class="block text-sm font-medium text-gray-700 mb-1">Description</label>
          <input class="w-full px-3 py-2 border rounded text-sm" type="text" placeholder="Movie.2026.1080p">
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Threads</label>
          <input class="w-24 px-3 py-2 border rounded text-sm" type="number" value="3" min="1" max="10">
        </div>
      </div>
      <div class="mt-3">
        <label class="block text-sm font-medium text-gray-700 mb-1">Uploader</label>
        <input class="w-full px-3 py-2 border rounded text-sm" type="text" value="rsfdl">
      </div>
    </div>
    <!-- Encryption -->
    <div>
      <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-3">Encryption (optional)</h3>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Password</label>
        <input class="w-full px-3 py-2 border rounded text-sm" type="password" placeholder="Leave empty for no encryption">
        <p class="text-xs text-gray-500 mt-1">AES-128-CBC encryption, same as SFDL.NET</p>
      </div>
    </div>
  </div>
</div>
