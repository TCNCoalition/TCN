   # Purpose
This document describes a maximally flexible, minimally prescriptive approach to interoperability between compatible Bluetooth Low Energy (BLE) proximity-based contact tracing apps.

# Brief summary of TCN (fka CEN) S2S (Server to Server) / C2S (Client to Server) interoperability
Each TCN app registers its memo field and API endpoint URL.
TCN coalition members decide which types/sources of reports to trust within their own ecosystems.
Each app developer can implement C2S and/or S2S interop for trusted reports.
In C2S, each app downloads reports directly from all trusted API endpoints.
In S2S, each app’s server downloads and filters reports and serves them to its app.
New TCN coalition members’ report types are vetted and added via config file.
Seamless UX: All interoperability work is hidden from the user.

# Background
In order to achieve the public health goals of digital contact tracing, there is an emerging need to ensure interoperability between all contact tracing apps that individuals in a given location might use. This means that some form of interoperability is required between government-sponsored contact tracing apps such as TraceTogether (Singapore) and Aarogya Setu (India), emerging private-sector contact tracing apps based on the TCN coalition’s TCN protocol or similar privacy-first decentralized protocols, and any other contact tracing systems that may gain widespread acceptance and use.

# Classification of contact tracing implementations
Contact tracing apps use two main technologies: location services (GPS, etc.) and proximity detection (Bluetooth Low Energy, BLE). This document considers interoperability between systems using BLE proximity detection.

Bluetooth proximity-based contact tracing apps can be classified into:
- Centralized implementations, where a (usually government-controlled) server receives information from clients on their interactions, performs exposure matching, and then notifies users of any matches. Centralized implementations known at the time of writing include TraceTogether (Singapore) and Aarogya Setu (India).
- Decentralized implementations, like those based on [the TCN protocol](https://github.com/TCNCoalition/TCN), where apps generate and broadcast short-lived cryptographically-generated temporary contact numbers (TCNs) over BLE, and users who later develop symptoms or test positive can send a report to any potential contacts by uploading a packet of data to a server containing a key to regenerate the previously used TCNs. The clients periodically download the keys uploaded by users reporting symptoms/tests and use them to determine whether any of the TCNs they’ve observed indicate an exposure to a potentially infectious individual. Other published decentralized protocols include [DP-3T](https://github.com/DP-3T/documents/blob/6ac18840fce3dd1c5e8f101dda7f036cffcbccee/DP3T%20White%20Paper.pdf).

# Parallel implementation of incompatible protocols
Direct interoperability between centralized contact tracing implementations would require cooperation between their designers. As the protocols powering such centralized implementations have not yet been published, we can only speculate as to (or [reverse engineer](https://medium.com/@frankvolkel/tracetogether-under-the-hood-7d5e509aeb5d)) what changes might be required. Such speculation is beyond the scope of this document. Furthermore, as open-source protocols gain traction, it is in the interest of other solutions to implement some form of interoperability.

Once their protocol is published, interoperability between a centralized system like TraceTogether and a distributed system like one based on the TCN protocol could be achieved by implementing an app that simply runs both protocols in parallel, exchanging information with the centralized server when it comes in contact with other TraceTogether users, and implementing the TCN protocol for all contacts with other TCN compatible apps.

# Interoperability between compatible TCN implementations
As outlined in [the DP-3T white paper](https://github.com/DP-3T/documents/blob/6ac18840fce3dd1c5e8f101dda7f036cffcbccee/DP3T%20White%20Paper.pdf), another likely interoperability scenario is between different decentralized implementations. For example, national health authorities in Europe, or state/local/private entities in the United States, may each produce different apps but desire them to be able to detect and communicate with each other.

If the apps are designed to be compatible with the TCN protocol (formerly known as the CEN, or Contact Event Number, protocol), each app that allows symptom or positive-test reporting would register its own app-specific memo type, as well as its own app API URL, in a registry/repository run by the TCN coalition.

Each app developer would then choose which other TCN coalition members’ reports they wish to use for exposure matching. It is expected that some apps that allow self-reporting of symptoms will choose to use all other TCN coalition members’ reports. Other apps, which only allow matching on signed reports of positive COVID-19 tests, would choose to only match on reports from the subset of TCN coalition members who have established relationships with local, regional, or national health authorities to authenticate such reports.

Having established which reports they want their app to process, the app developer would then decide whether to have each app independently download reports from the other TCN coalition members’ APIs, or to perform a server-side download of such reports. Performing server-side download allows an app developer to optionally perform any filtering required on the reports (for example to filter out those whose memo fields indicate self-reported symptoms), and then make all of the selected reports available to their own app via their own API.

When new TCN coalition members launch their own apps, or launch new partnerships with public health authorities to authenticate positive test results, other TCN coalition members can decide for themselves whether to trust and use the new reports. If they choose to do so, they can add support by simply updating their own servers’ or apps’ configuration files.

## Common TCN API

By implementing the following API, other apps who implement the TCN protocol will be able to fetch report data that was uploaded to other coalition member servers. **Note that the API doesn't enforce server-to-server nor client-to-server.** Rather it is flexible so those architectural decisions can be left to the team implementing the TCN protocol in their app.

Further documentation on the API can be found in the [API definition](https://github.com/Co-Epi/coepi-backend-aws/blob/master/api_definition/coepi_api_0.4.0.yml).
> Note: this link needs to be updated to point to a reference implementation on the TCN repo when we get there.

##### API Endpoints
| Method | Endpoint | Description |
| ------ | ------------ | ------------------------ |
| GET | /tcnreport | Returns a list of **signed** reports ('rvk \|\| tck_{j1-1} \|\| le_u16(j1) \|\| le_u16(j2) \|\| memo', refer to the [TCN protocol](https://github.com/TCNCoalition/TCN/blob/main/README.md)) concatenated together for a given time interval number. |

##### Query Parameters
| Name | Description |
| -------- | ---------------------- |
| intervalNumber | **Required:** Positive integer that corresponds to a specific fixed time interval and can be calculated as (unix_time / time_interval_ms) |
| date | **Optional:** Date in RFC3339 standard in UTC, without the time component. If not provided, default value of today's date is used |
| intervalLengthMs | **Optional:** The interval length in milliseconds used by the client to calculate intervalNumber. The server will respond with a 401 if the interval length does not match the expected length. |

# User experience

A TCN compatible app would be able to see all other compatible apps via Bluetooth, exchange TCNs, and check exposure via its own app API endpoint and/or those of other TCN coalition members. The app would notify the user of potential exposures identically whether the potentially infectious user was using the same app or another TCN compatible app. If the app user reported symptoms or a positive test, they would do so directly using their own app’s mechanisms (if implemented), and other apps would be able to retrieve that report via the mechanisms described above. This preserves each app’s UX, provides maximum flexibility to app developers, and ensures that all pertinent test or symptom reports are displayed to all users who should see them.

# Contributors

In addition to those shown in the GitHub commit history, the following individuals contributed to [drafting](https://docs.google.com/document/d/1B4Un1J04ZtwbY-xENkQFMLd9iDWm7ByjaPYumkTqotg/edit#) this document: Andreas Gebhard, Jenny Wanger, Nele Pauline, Harper Reed, and James Petrie
