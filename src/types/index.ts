export interface ButtonMapping {
  button_id: number;
  name: string;
  action_type: string; // 'key' | 'media' | 'macro' | 'layer_shift' | 'octashift'
  key_code: string;
  description: string;
}

export interface DeviceProfile {
  id: string;
  name: string;
  interface_type: string;
  serial_number: string;
  is_connected: boolean;
  active_layer: number;
  octashift_angle: number;
  layer_octashift_angles?: Record<number, number>;
  pointer_dpi: number;
  trackball_scroll_mode?: boolean;
  trackball_gesture_mode?: boolean;
  layer_names: Record<number, string>;
  button_mappings: Record<number, ButtonMapping[]>;
}

export interface AppConfig {
  autostart: boolean;
  minimize_to_tray: boolean;
  show_notifications: boolean;
  device: DeviceProfile;
}
