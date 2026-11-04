-- maps range_type 已默认给出

local mode = (range_type == "All") or (range_type ~= "Default")

local result = {}
local package_name = env.name() -- 获取当前选定进程名称

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
            local is_allocator = name:find("malloc", 1, true) or name:find("scudo", 1, true) or name == "[heap]"
            local is_empty = name == ""
            local is_app_related = name:find(package_name, 1, true) or name:find("/data/app", 1, true) or name:find("/data/data", 1, true)

            if is_allocator or is_empty then
                table.insert(result, area)
            elseif is_app_related then
                table.insert(result, area)

                local next_area = maps[i + 1]
                if next_area and next_area.name == "[anon:.bss]" then
                    table.insert(result, next_area)
                    i = i + 1
                end
            end
        end
    end

    i = i + 1
end

return result