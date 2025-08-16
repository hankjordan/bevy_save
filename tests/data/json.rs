pub const SNAPSHOT_V4: &str = r#"{
    "entities": {
        "4294967296": {
            "components": {}
        },
        "4294967297": {
            "components": {
                "format::Collect": {
                    "data": [
                        4294967299,
                        4294967300,
                        4294967301
                    ]
                },
                "format::Position": {
                    "x": 0.0,
                    "y": 1.0,
                    "z": 2.0
                },
                "format::Unit": {}
            }
        },
        "4294967298": {
            "components": {
                "format::Basic": {
                    "data": 4294967338
                },
                "format::Nullable": {
                    "data": 4294967373
                },
                "format::Unit": {}
            }
        },
        "4294967299": {
            "components": {
                "format::Position": {
                    "x": 6.0,
                    "y": 7.0,
                    "z": 8.0
                },
                "format::Unit": {}
            }
        },
        "4294967300": {
            "components": {
                "format::Nullable": {
                    "data": null
                }
            }
        }
    },
    "resources": {}
}"#;

pub const CHECKPOINTS_V3: &str = r#"{
    "entities": {
        "4294967296": {
            "components": {}
        },
        "4294967297": {
            "components": {
                "format::Collect": {
                    "data": [
                        4294967299,
                        4294967300,
                        4294967301
                    ]
                },
                "format::Position": {
                    "x": 0.0,
                    "y": 1.0,
                    "z": 2.0
                },
                "format::Unit": {}
            }
        },
        "4294967298": {
            "components": {
                "format::Basic": {
                    "data": 4294967338
                },
                "format::Nullable": {
                    "data": 4294967373
                },
                "format::Unit": {}
            }
        },
        "4294967299": {
            "components": {
                "format::Position": {
                    "x": 6.0,
                    "y": 7.0,
                    "z": 8.0
                },
                "format::Unit": {}
            }
        },
        "4294967300": {
            "components": {
                "format::Nullable": {
                    "data": null
                }
            }
        }
    },
    "resources": {},
    "rollbacks": {
        "checkpoints": [
            {
                "entities": {
                    "4294967296": {
                        "components": {}
                    },
                    "4294967297": {
                        "components": {
                            "format::Collect": {
                                "data": [
                                    4294967299,
                                    4294967300,
                                    4294967301
                                ]
                            },
                            "format::Position": {
                                "x": 0.0,
                                "y": 1.0,
                                "z": 2.0
                            },
                            "format::Unit": {}
                        }
                    },
                    "4294967298": {
                        "components": {
                            "format::Basic": {
                                "data": 4294967338
                            },
                            "format::Nullable": {
                                "data": 4294967373
                            },
                            "format::Unit": {}
                        }
                    },
                    "4294967299": {
                        "components": {
                            "format::Position": {
                                "x": 6.0,
                                "y": 7.0,
                                "z": 8.0
                            },
                            "format::Unit": {}
                        }
                    },
                    "4294967300": {
                        "components": {
                            "format::Nullable": {
                                "data": null
                            }
                        }
                    }
                },
                "resources": {}
            }
        ],
        "active": 0
    }
}"#;

pub const CHECKPOINTS_V4: &str = r#"{
    "entities": {
        "4294967296": {
            "components": {}
        },
        "4294967297": {
            "components": {
                "format::Collect": {
                    "data": [
                        4294967299,
                        4294967300,
                        4294967301
                    ]
                },
                "format::Position": {
                    "x": 0.0,
                    "y": 1.0,
                    "z": 2.0
                },
                "format::Unit": {}
            }
        },
        "4294967298": {
            "components": {
                "format::Basic": {
                    "data": 4294967338
                },
                "format::Nullable": {
                    "data": 4294967373
                },
                "format::Unit": {}
            }
        },
        "4294967299": {
            "components": {
                "format::Position": {
                    "x": 6.0,
                    "y": 7.0,
                    "z": 8.0
                },
                "format::Unit": {}
            }
        },
        "4294967300": {
            "components": {
                "format::Nullable": {
                    "data": null
                }
            }
        }
    },
    "resources": {
        "bevy_save::Checkpoints": {
            "snapshots": [
                {
                    "entities": {
                        "4294967296": {
                            "components": {}
                        },
                        "4294967297": {
                            "components": {
                                "format::Collect": {
                                    "data": [
                                        4294967299,
                                        4294967300,
                                        4294967301
                                    ]
                                },
                                "format::Position": {
                                    "x": 0.0,
                                    "y": 1.0,
                                    "z": 2.0
                                },
                                "format::Unit": {}
                            }
                        },
                        "4294967298": {
                            "components": {
                                "format::Basic": {
                                    "data": 4294967338
                                },
                                "format::Nullable": {
                                    "data": 4294967373
                                },
                                "format::Unit": {}
                            }
                        },
                        "4294967299": {
                            "components": {
                                "format::Position": {
                                    "x": 6.0,
                                    "y": 7.0,
                                    "z": 8.0
                                },
                                "format::Unit": {}
                            }
                        },
                        "4294967300": {
                            "components": {
                                "format::Nullable": {
                                    "data": null
                                }
                            }
                        }
                    },
                    "resources": {}
                }
            ],
            "active": 0
        }
    }
}"#;
