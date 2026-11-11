-- Heap growth of avante.utils.tokens.calculate_tokens on one realistic message.
-- Run from the repo root:
--   nvim --headless -u NONE -i NONE -l osspr-intake/harness/tokens_alloc.lua
-- Prints one integer: Lua heap bytes retained while the GC is stopped,
-- which is the allocation volume of that call (LuaJIT does not free the
-- intermediate strings until the collector runs).

local root = vim.fn.getcwd()
vim.opt.runtimepath:prepend(root)
package.path = root .. "/lua/?.lua;" .. root .. "/lua/?/init.lua;" .. package.path

local Tokens = require("avante.utils.tokens")

-- 16 KiB of whitespace-free base64, the shape of Avante image source.data.
-- calculate_tokens appends that blob onto the text part, then builds the
-- current token one byte at a time.
local IMAGE_BYTES = 16000
local ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
local image = string.rep(ALPHABET, math.ceil(IMAGE_BYTES / #ALPHABET)):sub(1, IMAGE_BYTES)
local content = {
  { type = "text", text = "Describe this screenshot of the failing test." },
  { type = "image", source = { data = image } },
}

local function allocation_bytes()
  collectgarbage("collect")
  collectgarbage("stop")
  local before = collectgarbage("count")
  local tokens = Tokens.calculate_tokens(content)
  local after = collectgarbage("count")
  collectgarbage("restart")
  collectgarbage("collect")
  if tokens ~= 7 then
    io.stderr:write("unexpected token count " .. tostring(tokens) .. "\n")
    os.exit(1)
  end
  return math.floor((after - before) * 1024 + 0.5)
end

allocation_bytes()
io.stdout:write(tostring(allocation_bytes()) .. "\n")
os.exit(0)
