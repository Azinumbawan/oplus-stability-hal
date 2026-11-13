// Reconstructed from vendor.oplus.hardware.stability.oplus_project-V1-service
// and vendor.oplus.hardware.stability.oplus_project-V1-ndk_platform.so.
//
// Method order is the exact AIDL transaction-code order (1..32) extracted from
// the BpOplusProject transact() calls in the NDK backend, plus transaction 33
// (get_dram_freq) which the device's own Java client declares
// (IOplusProject$Stub$Proxy: `const/16 v3, #int 33` + Parcel.readString())
// but the shipped native HAL does not implement. Do NOT reorder: AIDL
// assigns transaction codes by declaration order, so reordering breaks wire
// compatibility with the shipped HAL and with any prebuilt client.
package vendor.oplus.hardware.stability.oplus_project;

@VintfStability
interface IOplusProject {
    int get_eng_version();
    String get_force_reboot();
    int get_ftmmode();
    String get_hungtask();
    String get_mt_gpio();
    String get_ocp();
    String get_opboot();
    int get_operator_name();
    String get_oplusocp_status(in int arg0);
    String get_opluspoff_reason(in int arg0);
    String get_opluspon_reason(in int arg0);
    int get_pcb_version();
    String get_phoenix();
    String get_pmic_info();
    String get_poff_reason();
    String get_pon_reason();
    int get_project();
    int get_rf_type();
    int get_sau();
    String get_serial_ID();
    String get_shutdown_detect();
    boolean is_ufs_devices();
    String miscModule2Load();
    String miscModuleBlocklist();
    String miscReadLinkedModules();
    int[] readAgingData(in int arg0);
    String read_theia_node(in int arg0);
    boolean saveAgingData(in int arg0, in int[] arg1, in int arg2);
    boolean set_opboot(in String arg0);
    boolean set_phoenix(in String arg0);
    boolean set_shutdown_detect(in String arg0);
    boolean write_theia_node(in int arg0, in String arg1);
    String get_dram_freq();
}
