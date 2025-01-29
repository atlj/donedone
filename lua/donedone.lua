local log = require('log')

local donedone = {}

---@param comment string
local function add_entry(comment)
  local buffer = vim.api.nvim_get_current_buf()
  local file_path = vim.api.nvim_buf_get_name(buffer)
  local line = vim.fn.line(".")

  vim.system(
    {
      "dndn",
      "add",
      file_path,
      line,
      comment
    }
  )
end


---@param comment string?
function donedone.add_entry(comment)
  if comment ~= nil then
    add_entry(comment)
  else
    comment = vim.ui.input(
      { prompt = "Add a dndn entry" },
      ---@param input string?
      function(input)
        if input == nil then
          return
        end

        add_entry(input)
      end
    )
  end
end

function donedone.setup()
  vim.api.nvim_create_user_command(
    "DndnAddEntry",
    function()
      donedone.add_entry()
    end,
    {
      desc = "Add a new dndn entry."
    }
  )
end

return donedone
