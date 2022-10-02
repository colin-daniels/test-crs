pub enum RequestParts {}

// any uri-fragment that was received should only be retained in
// - m_uri
// - m_variableRequestURIRaw
// - m_variableRequestLine
//     size_t pos_raw_fragment = uri_s.find("#");
//     if (pos_raw_fragment != std::string::npos) {
//         uri_s = uri_s.substr(0, pos_raw_fragment);
//     }
//     m_uri_decoded = utils::uri_decode(uri_s);

//     std::string requestLine(std::string(method) + " " + std::string(uri)); ???
//     m_variableRequestLine.set(requestLine \
//         + " HTTP/" + std::string(http_version), m_variableOffset);
//
//         std::string qry = std::string(uri_s, pos_raw_query + 1,
//             uri_s.length() - (pos_raw_query + 1));
//         m_variableQueryString.set(qry, pos_raw_query + 1
//             + std::string(method).size() + 1);

//    std::string path_info;
//     if (pos_query == std::string::npos) {
//         path_info = std::string(m_uri_decoded, 0);
//     } else {
//         path_info = std::string(m_uri_decoded, 0, pos_query);
//     }
//     if (var_size == std::string::npos) {
//         var_size = uri_s.size();
//     }
//
//     m_variablePathInfo.set(path_info, m_variableOffset + strlen(method) +
//         1, var_size);
//     m_variableRequestFilename.set(path_info,  m_variableOffset +
//         strlen(method) + 1, var_size);

//     size_t offset = path_info.find_last_of("/\\");
//     if (offset != std::string::npos && path_info.length() > offset + 1) {
//         std::string basename = std::string(path_info, offset + 1,
//             path_info.length() - (offset + 1));
//         m_variableRequestBasename.set(basename, m_variableOffset +
//             strlen(method) + 1 + offset + 1);
//     }
