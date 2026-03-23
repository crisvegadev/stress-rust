/// Real coordinates of Mexican municipality seats (cabeceras municipales).
/// Each device picks a random municipality and drifts ±0.005° (~500m) per update.
/// This guarantees all points fall on land within Mexico.

pub struct Municipality {
    pub lat: f64,
    pub lng: f64,
}

/// 200 cabeceras municipales across all 32 states of Mexico.
pub const MUNICIPALITIES: &[Municipality] = &[
    // Aguascalientes
    Municipality { lat: 21.8818, lng: -102.2916 },
    Municipality { lat: 22.0096, lng: -102.3590 },
    Municipality { lat: 21.8463, lng: -102.7171 },
    // Baja California
    Municipality { lat: 32.5027, lng: -117.0037 },
    Municipality { lat: 30.7237, lng: -115.9946 },
    Municipality { lat: 32.6245, lng: -115.4523 },
    // Baja California Sur
    Municipality { lat: 24.1426, lng: -110.3128 },
    Municipality { lat: 23.0545, lng: -109.6976 },
    // Campeche
    Municipality { lat: 19.8301, lng: -90.5349 },
    Municipality { lat: 18.6513, lng: -91.8292 },
    Municipality { lat: 18.1813, lng: -89.7440 },
    // Chiapas
    Municipality { lat: 16.7528, lng: -93.1152 },
    Municipality { lat: 16.7370, lng: -92.6376 },
    Municipality { lat: 14.9019, lng: -92.2591 },
    Municipality { lat: 15.6958, lng: -93.2037 },
    Municipality { lat: 16.4514, lng: -92.4727 },
    // Chihuahua
    Municipality { lat: 28.6320, lng: -106.0889 },
    Municipality { lat: 31.6904, lng: -106.4245 },
    Municipality { lat: 28.7008, lng: -105.9685 },
    Municipality { lat: 27.5069, lng: -105.4510 },
    Municipality { lat: 28.1867, lng: -105.4724 },
    Municipality { lat: 26.9219, lng: -105.6717 },
    // Coahuila
    Municipality { lat: 25.4232, lng: -100.9924 },
    Municipality { lat: 26.9062, lng: -101.4220 },
    Municipality { lat: 25.3580, lng: -101.0466 },
    Municipality { lat: 27.8494, lng: -101.1346 },
    // Ciudad de México
    Municipality { lat: 19.4326, lng: -99.1332 },
    Municipality { lat: 19.3030, lng: -99.1500 },
    Municipality { lat: 19.3600, lng: -99.1643 },
    Municipality { lat: 19.4837, lng: -99.1478 },
    Municipality { lat: 19.3909, lng: -99.0866 },
    Municipality { lat: 19.3457, lng: -99.0288 },
    Municipality { lat: 19.2726, lng: -99.0942 },
    Municipality { lat: 19.2930, lng: -99.1612 },
    Municipality { lat: 19.3176, lng: -99.2269 },
    Municipality { lat: 19.3732, lng: -99.2618 },
    // Colima
    Municipality { lat: 19.2433, lng: -103.7247 },
    Municipality { lat: 19.0555, lng: -103.9596 },
    Municipality { lat: 19.1098, lng: -104.3350 },
    // Durango
    Municipality { lat: 24.0277, lng: -104.6532 },
    Municipality { lat: 24.4502, lng: -104.6650 },
    Municipality { lat: 23.9886, lng: -105.7510 },
    Municipality { lat: 25.0494, lng: -105.2543 },
    // Estado de México
    Municipality { lat: 19.2826, lng: -99.6557 },
    Municipality { lat: 19.4702, lng: -99.2346 },
    Municipality { lat: 19.5286, lng: -99.1960 },
    Municipality { lat: 19.3377, lng: -98.7412 },
    Municipality { lat: 19.6071, lng: -99.0520 },
    Municipality { lat: 19.3486, lng: -99.5629 },
    Municipality { lat: 19.2084, lng: -98.9830 },
    // Guanajuato
    Municipality { lat: 21.0190, lng: -101.2574 },
    Municipality { lat: 20.5845, lng: -100.3899 },
    Municipality { lat: 21.1250, lng: -101.6802 },
    Municipality { lat: 20.5234, lng: -100.8136 },
    Municipality { lat: 20.9676, lng: -101.4287 },
    Municipality { lat: 21.3563, lng: -101.9534 },
    // Guerrero
    Municipality { lat: 16.8634, lng: -99.8901 },
    Municipality { lat: 17.6620, lng: -101.5510 },
    Municipality { lat: 17.5536, lng: -99.5066 },
    Municipality { lat: 18.3050, lng: -99.5004 },
    Municipality { lat: 17.7917, lng: -101.2593 },
    // Hidalgo
    Municipality { lat: 20.1168, lng: -98.7330 },
    Municipality { lat: 20.2289, lng: -99.7410 },
    Municipality { lat: 20.0909, lng: -98.3590 },
    Municipality { lat: 20.3882, lng: -99.0114 },
    // Jalisco
    Municipality { lat: 20.6597, lng: -103.3496 },
    Municipality { lat: 20.6167, lng: -105.2282 },
    Municipality { lat: 20.3872, lng: -102.7708 },
    Municipality { lat: 21.3584, lng: -102.9099 },
    Municipality { lat: 20.2260, lng: -103.2464 },
    Municipality { lat: 20.7014, lng: -103.3908 },
    Municipality { lat: 19.7726, lng: -104.3620 },
    Municipality { lat: 20.7186, lng: -103.1729 },
    Municipality { lat: 20.5339, lng: -103.2431 },
    // Michoacán
    Municipality { lat: 19.7060, lng: -101.1949 },
    Municipality { lat: 19.9225, lng: -102.0832 },
    Municipality { lat: 20.3932, lng: -101.5296 },
    Municipality { lat: 19.1692, lng: -102.2044 },
    Municipality { lat: 18.9186, lng: -103.0466 },
    // Morelos
    Municipality { lat: 18.9186, lng: -99.2350 },
    Municipality { lat: 18.8490, lng: -99.0145 },
    Municipality { lat: 18.7683, lng: -98.9586 },
    // Nayarit
    Municipality { lat: 21.5039, lng: -104.8946 },
    Municipality { lat: 20.7395, lng: -105.2952 },
    Municipality { lat: 21.7624, lng: -105.1876 },
    // Nuevo León
    Municipality { lat: 25.6866, lng: -100.3161 },
    Municipality { lat: 25.7506, lng: -100.2881 },
    Municipality { lat: 25.6702, lng: -100.2437 },
    Municipality { lat: 25.6487, lng: -100.0958 },
    Municipality { lat: 25.7968, lng: -100.3786 },
    Municipality { lat: 25.4210, lng: -100.0011 },
    Municipality { lat: 25.5685, lng: -100.2445 },
    Municipality { lat: 24.7862, lng: -100.0010 },
    // Oaxaca
    Municipality { lat: 17.0731, lng: -96.7266 },
    Municipality { lat: 16.4408, lng: -95.0072 },
    Municipality { lat: 15.8546, lng: -96.4458 },
    Municipality { lat: 17.9621, lng: -96.7180 },
    Municipality { lat: 15.6695, lng: -96.1340 },
    Municipality { lat: 16.3194, lng: -95.2395 },
    // Puebla
    Municipality { lat: 19.0414, lng: -98.2063 },
    Municipality { lat: 18.4667, lng: -97.3928 },
    Municipality { lat: 20.5364, lng: -97.4080 },
    Municipality { lat: 18.9136, lng: -97.7905 },
    Municipality { lat: 20.0590, lng: -97.4783 },
    // Querétaro
    Municipality { lat: 20.5888, lng: -100.3899 },
    Municipality { lat: 20.5469, lng: -100.1830 },
    Municipality { lat: 21.2402, lng: -99.6282 },
    // Quintana Roo
    Municipality { lat: 21.1619, lng: -86.8515 },
    Municipality { lat: 20.5058, lng: -87.3736 },
    Municipality { lat: 18.5036, lng: -88.3053 },
    Municipality { lat: 19.5967, lng: -88.0396 },
    // San Luis Potosí
    Municipality { lat: 22.1565, lng: -100.9855 },
    Municipality { lat: 22.2397, lng: -99.9949 },
    Municipality { lat: 21.9907, lng: -99.6214 },
    Municipality { lat: 22.0995, lng: -100.7675 },
    // Sinaloa
    Municipality { lat: 24.8049, lng: -107.3940 },
    Municipality { lat: 23.2494, lng: -106.4112 },
    Municipality { lat: 25.7913, lng: -108.9856 },
    Municipality { lat: 24.7662, lng: -107.3851 },
    Municipality { lat: 25.4689, lng: -108.3774 },
    // Sonora
    Municipality { lat: 29.0729, lng: -110.9559 },
    Municipality { lat: 27.9239, lng: -110.8975 },
    Municipality { lat: 31.3156, lng: -110.9420 },
    Municipality { lat: 30.7253, lng: -112.0740 },
    Municipality { lat: 27.4859, lng: -109.9417 },
    // Tabasco
    Municipality { lat: 17.9892, lng: -92.9474 },
    Municipality { lat: 18.2229, lng: -93.2141 },
    Municipality { lat: 17.7779, lng: -92.5698 },
    Municipality { lat: 18.3944, lng: -92.6498 },
    // Tamaulipas
    Municipality { lat: 23.7369, lng: -99.1411 },
    Municipality { lat: 25.8696, lng: -97.5025 },
    Municipality { lat: 22.2331, lng: -97.8511 },
    Municipality { lat: 27.4760, lng: -99.5075 },
    Municipality { lat: 24.7886, lng: -98.7782 },
    Municipality { lat: 23.7200, lng: -99.1700 },
    // Tlaxcala
    Municipality { lat: 19.3182, lng: -98.2375 },
    Municipality { lat: 19.2825, lng: -98.4125 },
    Municipality { lat: 19.5221, lng: -98.1637 },
    // Veracruz
    Municipality { lat: 19.1738, lng: -96.1342 },
    Municipality { lat: 19.5316, lng: -96.9174 },
    Municipality { lat: 18.4500, lng: -95.0984 },
    Municipality { lat: 20.9670, lng: -97.4010 },
    Municipality { lat: 17.9964, lng: -94.5434 },
    Municipality { lat: 21.1419, lng: -98.4144 },
    Municipality { lat: 18.8500, lng: -97.1000 },
    // Yucatán
    Municipality { lat: 20.9674, lng: -89.5926 },
    Municipality { lat: 20.8686, lng: -89.0175 },
    Municipality { lat: 20.3947, lng: -89.9948 },
    Municipality { lat: 21.0833, lng: -89.7167 },
    Municipality { lat: 20.7311, lng: -89.6975 },
    // Zacatecas
    Municipality { lat: 22.7709, lng: -102.5832 },
    Municipality { lat: 23.1900, lng: -102.8700 },
    Municipality { lat: 22.1237, lng: -103.2506 },
    Municipality { lat: 22.4564, lng: -102.7124 },
    // Extra — remaining large cities for variety
    Municipality { lat: 20.9684, lng: -89.5925 }, // Mérida
    Municipality { lat: 19.1787, lng: -96.1403 }, // Veracruz puerto
    Municipality { lat: 24.0226, lng: -104.6588 }, // Durango city
    Municipality { lat: 17.0542, lng: -96.7216 }, // Oaxaca city
    Municipality { lat: 19.8437, lng: -90.5258 }, // Campeche city
    Municipality { lat: 21.1526, lng: -86.8462 }, // Cancún
    Municipality { lat: 20.6271, lng: -87.0802 }, // Tulum
    Municipality { lat: 16.9175, lng: -99.8442 }, // Acapulco detail
    Municipality { lat: 20.2107, lng: -87.4280 }, // Playa del Carmen
    Municipality { lat: 19.7005, lng: -101.1847 }, // Morelia detail
    Municipality { lat: 21.4869, lng: -104.8849 }, // Tepic detail
    Municipality { lat: 19.1793, lng: -104.2839 }, // Manzanillo
    Municipality { lat: 20.5218, lng: -100.8126 }, // Celaya
    Municipality { lat: 16.8634, lng: -99.8237 }, // Acapulco norte
    Municipality { lat: 18.6500, lng: -99.1000 }, // Cuautla
    Municipality { lat: 25.5428, lng: -103.4068 }, // Torreón
    Municipality { lat: 19.3219, lng: -99.1594 }, // Coyoacán
    Municipality { lat: 19.3611, lng: -99.0690 }, // Iztapalapa
    Municipality { lat: 19.4424, lng: -99.0204 }, // Ciudad Neza
    Municipality { lat: 19.5120, lng: -99.2355 }, // Azcapotzalco
    Municipality { lat: 19.2465, lng: -99.1013 }, // Xochimilco
    Municipality { lat: 19.5333, lng: -99.3000 }, // Naucalpan
    Municipality { lat: 19.3833, lng: -99.0333 }, // Iztacalco
    Municipality { lat: 19.3500, lng: -99.1833 }, // Benito Juárez
    Municipality { lat: 19.4200, lng: -99.1800 }, // Cuauhtémoc
    Municipality { lat: 19.4000, lng: -99.2000 }, // Miguel Hidalgo
    Municipality { lat: 20.0553, lng: -99.3434 }, // Tula de Allende
    Municipality { lat: 20.4631, lng: -99.2251 }, // Ixmiquilpan
    Municipality { lat: 19.7141, lng: -99.6597 }, // Atlacomulco
    Municipality { lat: 19.3218, lng: -99.6604 }, // Toluca detail
];
