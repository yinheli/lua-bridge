

-- do other staff
print("Hello, world!")

print("ver:" .. _version)

function handle(ctx)
  while true do
    -- use default buf size, or pass size as required
    local from, data = ctx:select(0)
    -- if no data, break
    if #data == 0 then
      break
    end
    if from == 'client' then
      info("client: " .. bytes_to_string(data))
      ctx:write_backend(data)
    elseif from == 'backend' then
      info("backend: " .. bytes_to_string(data))
      ctx:write_client(data)
    end
  end
end
