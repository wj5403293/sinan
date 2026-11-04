-- maps filter_modules 已默认给出

local function basename(path)
    return path:match("^.+/(.+)$") or path
end

local function should_filter(name)
    if filter_modules and #filter_modules > 0 and filter_modules[1] ~= "all" then
        for _, m in ipairs(filter_modules) do
            if name:find(m, 1, true) then
                return true
            end
        end
        
        return false
    end
    
    return true
end

local result = {}

local i = 1
while i <= #maps do
    local current = maps[i]

    if not current.is_read
       or (not current.name:find("/data/app", 1, true) 
           and not current.name:find("/data/data", 1, true))
       or not should_filter(current.name) then
        i = i + 1
        goto continue
    end

    local start_idx = i
    local end_idx = i
    local j = i

    while j <= #maps do
        local next_area = maps[j]

        if next_area.name == current.name then
            end_idx = j
        elseif (next_area.name == "" and maps[j - 1].name == current.name) or next_area.name == "[anon:.bss]" then
            end_idx = j
        elseif next_area.is_read and next_area.name ~= "" then
            break
        end
        
        j = j + 1
    end

    if end_idx > start_idx then
        table.insert(result, {
            start = maps[start_idx].start,
            ["end"] = maps[end_idx]["end"],
            name = basename(maps[start_idx].name),
        })
        
        i = j - 1
    end

    i = i + 1
    ::continue::
end

return result