-- maps range_type 已默认给出

local mode = (range_type == "All") or (range_type ~= "Default")

local result = {}
local process_name = env.name()

if process_name then
    process_name = process_name:lower()
end

local i = 1
while i <= #maps do
    local area = maps[i]

    if mode then
        if area.is_read or area.is_write then
            table.insert(result, area)
        end
    else
        if area.is_read then
            local name = area.name
            local name_lower = name:lower()
            
            local is_empty = name == ""
            
            local is_app_related = false
            if process_name and process_name ~= "" then
                is_app_related = name_lower:find(process_name, 1, true)
            end

            if is_empty or is_app_related then
                table.insert(result, area)
            end
        end
    end

    i = i + 1
end

return result